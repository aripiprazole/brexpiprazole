use asena_hir_derive::*;

use crate::{literal::HirLiteral, pattern::HirPatternId, stmt::HirStmtId, value::HirValueId, *};

#[derive(Hash, Clone, Debug)]
#[hir_node(transparent)]
pub struct HirExprLiteral(pub HirLiteral);

#[derive(Hash, Clone, Debug)]
#[hir_node(bridge)]
pub struct HirExprGroup {
    pub value: HirValueId,
}

#[derive(Hash, Clone, Debug)]
#[hir_node]
pub struct HirDsl {
    pub parameters: Vec<NameId>,
    pub stmts: Vec<HirStmtId>,
}

#[derive(Hash, Clone, Debug)]
#[hir_node]
pub struct HirExprCall {
    pub callee: HirValueId,
    pub arguments: Vec<HirValueId>,
    pub as_dsl: Option<HirDsl>,
}

#[derive(Hash, Clone, Debug)]
#[hir_node]
pub struct HirExprReference {
    pub scope: ScopeId,
    pub name: NameId,
}

#[derive(Hash, Clone, Debug)]
#[hir_node]
pub struct HirExprMatch {
    pub scrutinee: HirValueId,
    pub cases: Vec<data::HirMatchCase>,
}

#[derive(Hash, Clone, Debug)]
#[hir_node]
pub struct HirExprHelp {
    pub value: HirValueId,
}

#[derive(Hash, Clone, Debug)]
#[hir_node]
pub struct HirExprAnn {
    pub value: HirValueId,
    pub against: HirTypeId,
}

#[derive(Hash, Clone, Debug)]
#[hir_node]
pub struct HirExprLam {
    pub parameters: Vec<NameId>,
    pub value: HirExprId,
}

#[derive(Hash, Clone, Debug)]
#[hir_node]
pub struct HirExprIf {
    pub condition: HirValueId,
    pub then_branch: data::HirIfBranch,
    pub otherwise_branch: Option<data::HirIfBranch>,
}

#[derive(Hash, Clone, Debug)]
#[hir_node]
pub struct HirExprArray {
    pub items: Vec<HirValueId>,
}

#[derive(Hash, Clone, Debug)]
#[hir_kind]
pub enum HirExprKind {
    Error,
    Unit,
    This,
    Group(HirExprGroup),
    Literal(HirExprLiteral),
    Reference(HirExprReference),
    Call(HirExprCall),
    Match(HirExprMatch),
    Help(HirExprHelp),
    Ann(HirExprAnn),
    Lam(HirExprLam),
    If(HirExprIf),
    Array(HirExprArray),
}

#[derive(Hash, Clone, Debug)]
#[hir_struct(HirVisitor)]
pub struct HirExpr {
    pub span: asena_span::Loc,
    pub id: HirExprId,
    pub kind: HirExprKind,
}

/// Data structures module split into its own module to better disposition, as
/// it is a bit large, and it's used as extension to [`Expr`].
pub mod data {
    use super::*;

    #[derive(Hash, Clone, Debug)]
    pub enum HirIfBranch {
        Expr(HirValueId),
        Block(Vec<HirStmtId>),
    }

    #[derive(Hash, Clone, Debug)]
    pub enum HirMatchArm {
        Expr(HirValueId),
        Block(HirStmtId),
    }

    #[derive(Hash, Clone, Debug)]
    pub struct HirMatchCase {
        pub pattern: HirPatternId,
        pub value: HirMatchArm,
    }
}
