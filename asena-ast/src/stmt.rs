use asena_derive::{node_leaf, Leaf};
use asena_leaf::ast::{Cursor, GreenTree};
use asena_leaf::ast_enum;
use asena_leaf::node::TreeKind;

use asena_span::Spanned;

use crate::*;

#[derive(Leaf, Clone)]
pub struct Ask(GreenTree);

impl Ask {
    #[node_leaf]
    pub fn pattern(&self) -> Cursor<Pat> {
        todo!()
    }

    #[node_leaf]
    pub fn value(&self) -> Cursor<Expr> {
        todo!()
    }
}

#[derive(Leaf, Clone)]
pub struct Set(GreenTree);

impl Set {
    #[node_leaf]
    pub fn pattern(&self) -> Cursor<Pat> {
        self.filter::<Pat>().first()
    }

    #[node_leaf]
    pub fn value(&self) -> Cursor<Expr> {
        self.filter::<Expr>().first()
    }
}

#[derive(Leaf, Clone)]
pub struct Return(GreenTree);

impl Return {
    /// This is using directly [ExprRef] in the AST, because when expanded, this will generate
    /// and [Option] wrapped value.
    #[node_leaf]
    pub fn value(&self) -> Cursor<Expr> {
        todo!()
    }
}

#[derive(Leaf, Clone)]
pub struct Eval(GreenTree);

impl Eval {
    #[node_leaf]
    pub fn value(&self) -> Cursor<Expr> {
        self.filter::<Expr>().first()
    }
}

ast_enum! {
    pub enum Stmt {
        Ask    <- TreeKind::StmtAsk,    // <local_id> <- <expr>
        Set    <- TreeKind::StmtLet,    // let <local_id> = <expr>
        Return <- TreeKind::StmtReturn, // return <expr?>
        Eval   <- TreeKind::StmtExpr,   // <expr?>
    }
}

pub type StmtRef = Spanned<Stmt>;
