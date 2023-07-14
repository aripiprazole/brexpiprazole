use asena_ast::{Ann, App, Array, Dsl, If, Lam, Let, LocalExpr, Match};
use asena_hir::{
    expr::data::{HirDsl, HirMatchCase, HirMatchKind},
    pattern::HirPattern,
    stmt::HirStmt,
};

use crate::{db::AstLowerrer, literal::make_literal};

use super::*;

pub struct ExprLowering<'a> {
    pub db: &'a dyn AstLowerrer,
    pub instructions: Vec<HirStmt>,
}

impl<'a> ExprLowering<'a> {
    pub fn new(db: &'a dyn AstLowerrer) -> Self {
        ExprLowering {
            db,
            instructions: vec![],
        }
    }

    pub fn make(&mut self, expr: Expr) -> HirExpr {
        let kind = match expr {
            Expr::Group(ref group) => HirExprKind::from(HirExprGroup {
                value: self.db.lower_value(group.value()),
            }),
            Expr::Help(ref expr) => HirExprKind::from(HirExprHelp {
                value: self.db.lower_value(expr.value()),
            }),
            Expr::LiteralExpr(ref expr) => {
                let literal = make_literal(expr.literal().data().clone());

                HirExprKind::from(HirExprLiteral(literal))
            }

            Expr::Error => HirExprKind::Error,
            Expr::SelfExpr(_) => HirExprKind::This,
            Expr::Unit(_) => HirExprKind::Unit,
            Expr::Dsl(ref expr) => self.make_dsl(expr),
            Expr::Let(ref expr) => self.make_let(expr),
            Expr::LocalExpr(ref expr) => self.make_local(expr),
            Expr::Ann(ref ann) => self.make_ann(ann),
            Expr::Array(ref array) => self.make_array(array),
            Expr::App(ref expr) => self.make_app(expr),
            Expr::Infix(ref expr) => self.make_infix(expr),
            Expr::If(ref expr) => self.make_if(expr),
            Expr::Match(ref expr) => self.make_match(expr),
            Expr::Lam(ref expr) => self.make_lam(expr),

            // dependent types unsupported syntax
            ref expr @ Expr::Qual(_) | ref expr @ Expr::Pi(_) | ref expr @ Expr::Sigma(_) => {
                self.db
                    .reporter()
                    .report(expr, UnsupportedDependentTypesError);

                HirExprKind::Error
            }
        };

        self.db.intern_expr(HirExprData {
            kind,
            span: make_location(self.db, &expr),
        })
    }

    fn make_let(&self, _expr: &Let) -> HirExprKind {
        todo!("lowering let expressions is not yet implemented")
    }

    fn make_dsl(&self, expr: &Dsl) -> HirExprKind {
        let mut hir_call = match expr.callee() {
            Expr::App(ref app) => match self.make_app(app) {
                HirExprKind::Call(call) => call,
                _ => unreachable!(),
            },
            _ => HirExprCall {
                // TODO: handle Do, etc
                callee: HirCallee::Value(self.db.lower_value(expr.callee())),
                arguments: vec![],
                as_dsl: None,
            },
        };

        hir_call.as_dsl = Some(HirDsl {
            parameters: vec![], // TODO
            value: self.db.lower_block(expr.block()),
        });

        HirExprKind::from(hir_call)
    }

    fn make_local(&self, expr: &LocalExpr) -> HirExprKind {
        let name = self.db.intern_name(expr.to_fn_id().to_string());

        HirExprKind::from(HirExprReference { name })
    }

    fn make_ann(&self, expr: &Ann) -> HirExprKind {
        let value = self.db.lower_value(expr.value());
        let against = self.db.lower_type(expr.against());

        HirExprKind::from(HirExprAnn { value, against })
    }

    fn make_array(&self, array: &Array) -> HirExprKind {
        let items = array
            .items()
            .into_iter()
            .map(|e| self.db.lower_value(e))
            .collect();

        HirExprKind::from(HirExprArray { items })
    }

    fn make_app(&self, app: &App) -> HirExprKind {
        let callee = self.db.lower_value(app.callee());
        let argument = self.db.lower_value(app.argument());

        HirExprKind::from(HirExprCall {
            callee: HirCallee::Value(callee),
            arguments: vec![argument],
            as_dsl: None,
        })
    }

    fn make_infix(&self, infix: &Infix) -> HirExprKind {
        let lhs = self.db.lower_value(infix.lhs());
        let rhs = self.db.lower_value(infix.rhs());

        let callee = match infix.fn_id().as_str() {
            "+" => HirCallee::Add,
            "-" => HirCallee::Sub,
            "*" => HirCallee::Mul,
            "/" => HirCallee::Div,
            fn_id => self.make_callee(infix, fn_id),
        };

        HirExprKind::from(HirExprCall {
            callee,
            arguments: vec![lhs, rhs],
            as_dsl: None,
        })
    }

    fn make_if(&self, expr: &If) -> HirExprKind {
        HirExprKind::from(HirExprMatch {
            scrutinee: self.db.lower_value(expr.cond()),
            cases: hashset![
                HirMatchCase {
                    pattern: HirPattern::new_true(self.db),
                    value: self.db.lower_branch(expr.then_branch()),
                },
                HirMatchCase {
                    pattern: HirPattern::new_false(self.db),
                    value: self.db.lower_branch(expr.else_branch()),
                }
            ],
            kind: HirMatchKind::If,
        })
    }

    fn make_match(&self, expr: &Match) -> HirExprKind {
        let scrutinee = self.db.lower_value(expr.scrutinee());
        let cases = expr
            .cases()
            .into_iter()
            .map(|case| self.lower_case(case))
            .collect();

        HirExprKind::from(HirExprMatch {
            scrutinee,
            cases,
            kind: HirMatchKind::Match,
        })
    }

    fn make_lam(&self, expr: &Lam) -> HirExprKind {
        let value = self.db.lower_value(expr.value());
        let parameters = expr
            .parameters()
            .iter()
            .map(|parameter| self.db.intern_name(parameter.name().to_fn_id().to_string()))
            .collect_vec();

        HirExprKind::from(HirExprLam { parameters, value })
    }

    fn make_callee(&self, infix: &Infix, fn_id: &str) -> HirCallee {
        let span = make_location(self.db, infix);
        let name = self.db.intern_name(fn_id.into());

        let expr = self.db.intern_expr(HirExprData {
            kind: HirExprKind::from(HirExprReference { name }),
            span,
        });

        let value = self.db.intern_value(HirValueData {
            kind: HirValueKind::from(HirValueExpr(expr)),
            span,
        });

        HirCallee::Value(value)
    }

    fn lower_case(&self, case: asena_ast::Case) -> HirMatchCase {
        let pattern = self.db.lower_pattern(case.pat());
        let value = self.db.lower_branch(case.value());

        HirMatchCase { pattern, value }
    }
}
