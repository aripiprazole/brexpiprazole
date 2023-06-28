use std::{marker::PhantomData, rc::Rc};

use super::*;

/// A cursor is a reference to a node in the tree.
///
/// It is used to traverse the tree, and to modify it.
pub struct Cursor<T> {
    pub(crate) value: Rc<RefCell<GreenTree>>,
    _marker: PhantomData<T>,
}

impl<T: Leaf> Cursor<T> {
    /// Creates a new cursor without any value.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Updates the value of the current cursor with a new [Cursor].
    pub fn set(&self, value: Cursor<T>) {
        self.value.replace(value.value.borrow().clone());
    }

    /// Creates a new cursor with a reference to the `concrete syntax tree`, using
    /// the wrapper [GreenTree].
    pub fn new<I: Into<GreenTree>>(value: I) -> Self {
        Self {
            value: Rc::new(RefCell::new(value.into())),
            _marker: PhantomData,
        }
    }

    /// Deeply duplicates the current cursor and returns a new [Cursor] instance.
    pub fn as_new_node(&self) -> Self {
        Self {
            _marker: PhantomData,
            value: Rc::new(RefCell::new(self.value.borrow().clone())),
        }
    }
}

impl<T: Node + Leaf> Cursor<T> {
    /// Updates the value of the current cursor with a new [T].
    pub fn replace(&self, value: T) {
        self.value.replace(value.unwrap());
    }

    pub fn location(&self) -> Spanned<T>
    where
        T: Located + 'static,
    {
        match &*self.value.borrow() {
            GreenTree::Leaf { data, .. } => data.replace(T::new(data.clone())),
            GreenTree::Token(lexeme) => {
                let Some(value) = lexeme.downcast_ref::<T>() else {
                    return Spanned::default();
                };

                lexeme.token.clone().swap(value.clone())
            }
            _ => Spanned::default(),
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(&*self.value.borrow(), GreenTree::None)
    }

    /// Returns the current cursor if it's not empty, otherwise returns false.
    pub fn is_empty(&self) -> bool {
        match &*self.value.borrow() {
            GreenTree::Leaf { data, .. } => !data.children.is_empty(),
            GreenTree::Vec(children) => !children.is_empty(),
            _ => false,
        }
    }

    /// Creates a new cursor with the given value.
    pub fn of(value: T) -> Self {
        Self {
            value: Rc::new(RefCell::new(value.unwrap())),
            _marker: PhantomData,
        }
    }

    /// Returns the current cursor if it's not empty, otherwise returns a default value.
    pub fn as_leaf(&self) -> T {
        T::new(self.value.borrow().clone())
    }
}

impl<T: Leaf> Default for Cursor<T> {
    fn default() -> Self {
        Self {
            value: Rc::new(RefCell::new(Default::default())),
            _marker: PhantomData,
        }
    }
}

impl<T: Node + Leaf> Cursor<Vec<T>> {
    pub fn first(self) -> Cursor<T> {
        self.as_leaf().first().cloned().into()
    }

    pub fn skip(self, n: usize) -> Cursor<Vec<T>> {
        self.as_leaf()
            .iter()
            .skip(n)
            .cloned()
            .collect::<Vec<_>>()
            .into()
    }
}

impl<T: Node + Leaf> From<Vec<T>> for Cursor<Vec<T>> {
    fn from(value: Vec<T>) -> Self {
        Cursor::of(value)
    }
}

impl<T: Node + Leaf> Node for Vec<T> {
    fn new<I: Into<GreenTree>>(tree: I) -> Self {
        let tree: GreenTree = tree.into();

        match tree {
            GreenTree::Vec(values) => values
                .into_iter()
                .map(|value| T::new(value))
                .collect::<Vec<_>>(),
            GreenTree::Leaf { data, .. } => data
                .children
                .iter()
                .map(|child| match child.value {
                    Child::Tree(ref tree) => T::new(child.replace(tree.clone())),
                    Child::Token(ref token) => {
                        T::terminal(child.replace(token.clone())).unwrap_or_default()
                    }
                })
                .collect::<Vec<_>>(),
            _ => vec![],
        }
    }

    fn unwrap(self) -> GreenTree {
        GreenTree::Vec(self.into_iter().map(|x| x.unwrap()).collect::<Vec<_>>())
    }
}

impl<T: Node + Leaf> From<Option<T>> for Cursor<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => Self::new(value.unwrap()),
            None => Self {
                value: Rc::new(RefCell::new(GreenTree::None)),
                _marker: PhantomData,
            },
        }
    }
}

impl<T: Node + Leaf> Display for Cursor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_leaf())
    }
}

impl<T: Node + Leaf> Debug for Cursor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cursor({:?})", self.as_leaf())
    }
}

impl<T: Leaf> From<GreenTree> for Cursor<T> {
    fn from(value: GreenTree) -> Self {
        Cursor::new(value)
    }
}

impl<T: Leaf> Clone for Cursor<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: Default + Leaf + Node + 'static> FromResidual for Cursor<T> {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        match residual {
            Some(_) => unreachable!(),
            None => Cursor::empty(),
        }
    }
}

impl<T: Default + Leaf + Node + 'static> Try for Cursor<T> {
    type Output = T;

    type Residual = Option<std::convert::Infallible>;

    fn from_output(output: Self::Output) -> Self {
        Self::of(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        let tree = self.value.borrow();
        match &*tree {
            GreenTree::Token(lexeme) => match lexeme.downcast_ref::<T>() {
                Some(value) => ControlFlow::Continue(value.clone()),
                None => ControlFlow::Break(None),
            },
            GreenTree::Empty => ControlFlow::Break(None),
            GreenTree::None => ControlFlow::Break(None),
            _ => ControlFlow::Continue(T::new(tree.clone())),
        }
    }
}
