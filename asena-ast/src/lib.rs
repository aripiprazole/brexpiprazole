use asena_derive::{ast_debug, ast_leaf, ast_of, Leaf};
use asena_leaf::ast::{GreenTree, Walkable};
use std::fmt::{Debug, Formatter};

/// Represents a true-false value, just like an wrapper to [bool], this represents if an integer
/// value is signed, or unsigned.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Signed {
    Signed,
    Unsigned,
}

/// Represents the root of the asena source code file, it contains a set of declarations.
#[derive(Default, Leaf, Clone)]
pub struct AsenaFile(GreenTree);

#[ast_of]
#[ast_debug]
impl AsenaFile {
    #[ast_leaf]
    pub fn declarations(&self) -> Vec<Decl> {
        self.filter()
    }
}

pub trait FileWalker:
    DeclWalker + BodyWalker + PropertyWalker + ExprWalker + PatWalker + StmtWalker + CommandWalker
{
    fn walk_file(&mut self, _value: &AsenaFile) {}
}

/// NOTE: implemented on hand, because of the [AsenaFile] is a root node, and it's not a [Decl],
/// and it would be painful to list every walker trait on the macro call.
impl<W: FileWalker> Walkable<W> for AsenaFile
where
    W: DeclWalker,
    W: BodyWalker,
    W: PropertyWalker,
    W: ExprWalker,
    W: PatWalker,
    W: StmtWalker,
    W: CommandWalker,
{
    fn walk(&self, walker: &mut W) {
        for decl in self.declarations().iter() {
            decl.walk(walker);
        }
        walker.walk_file(self);
    }
}

pub use binding::*;
pub use body::*;
use decl::command::CommandWalker;
pub use decl::*;
pub use expr::*;
pub use identifier::*;
pub use literal::*;
pub use parameter::*;
pub use pat::*;
pub use stmt::*;
pub use traits::binary::*;

pub mod binding;
pub mod body;
pub mod decl;
pub mod expr;
pub mod identifier;
pub mod literal;
pub mod parameter;
pub mod pat;
pub mod stmt;
pub mod walker;

pub mod traits {
    pub mod binary;
}
