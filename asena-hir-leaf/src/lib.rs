#![feature(associated_type_bounds)]

use std::{fmt::Debug, hash::Hash};

pub mod hir_debug;
pub mod hir_sexpr;

pub use hir_debug::*;

pub type HirLoc = asena_span::Loc;

pub trait HirId: Debug + Copy + Hash {
    type Node: HirNode;

    fn new(node: Self::Node) -> Self;
}

pub trait HirNode: From<<Self::Id as HirId>::Node> {
    type Id: HirId<Node: Into<Self>>;
    type Visitor<'a, T>: ?Sized;

    fn new(id: Self::Id) -> Self;

    fn accept<O: Default>(&mut self, visitor: &mut Self::Visitor<'_, O>) -> O;
}

pub trait HirLocated {
    fn location(&self) -> HirLoc;
}

pub struct HirBorrow<'a, N: HirNode> {
    pub node: &'a N,
}

pub trait HirBaseDatabase {
    /// Finds the node in the given database.
    fn node<N: HirNode>(&self, id: N::Id) -> Option<HirBorrow<'_, N>>
    where
        Self: Sized;
}
