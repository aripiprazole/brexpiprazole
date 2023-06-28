use std::{marker::PhantomData, rc::Rc};

use crate::node::TreeKind;

use super::*;

/// A cursor is a reference to a node in the tree.
///
/// It is used to traverse the tree, and to modify it.
pub struct Cursor<T> {
    pub(crate) value: Rc<RefCell<Value<T>>>,
    _marker: PhantomData<T>,
}

#[derive(Clone)]
pub enum Value<T> {
    Green(GreenTree),
    Ref(Rc<T>),
}

impl<T: Node + Clone> Value<T> {
    pub fn green(&self) -> GreenTree {
        match self {
            Value::Green(value) => value.clone(),
            Value::Ref(value) => (**value).clone().unwrap(),
        }
    }
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
            value: Rc::new(RefCell::new(Value::Green(value.into()))),
            _marker: PhantomData,
        }
    }

    /// Deeply duplicates the current cursor and returns a new [Cursor] instance.
    pub fn as_new_node(&self) -> Self {
        Self {
            _marker: PhantomData,
            value: match &*self.value.borrow() {
                Value::Green(value) => Rc::new(RefCell::new(Value::Green(value.clone()))),
                Value::Ref(value) => Rc::new(RefCell::new(Value::Ref(value.clone()))),
            },
        }
    }

    /// Updates the value of the current cursor with a new [T].
    pub fn replace(&self, value: T) {
        self.value.replace(Value::Ref(Rc::new(value)));
    }

    /// Updates the value of the current cursor with a new [T].
    pub fn replace_rc(&self, value: Rc<T>) {
        self.value.replace(Value::Ref(value));
    }
}

impl<T: Node + Leaf> Cursor<T> {
    pub fn location(&self) -> Spanned<T>
    where
        T: Located + 'static,
    {
        match self.value.borrow().green() {
            GreenTree::Leaf { data, .. } => data.replace(T::new(data.clone())),
            GreenTree::Token(lexeme) => {
                let Some(value) = lexeme.downcast_ref::<T>() else {
                    return Spanned::default();
                };

                lexeme.token.clone().swap(value.clone())
            }
            GreenTree::Empty => Spanned::default(),
            GreenTree::None => Spanned::default(),
        }
    }

    pub fn is_none(&self) -> bool {
        match self.value.borrow().green() {
            GreenTree::Leaf { .. } => false,
            GreenTree::Token(..) => false,
            GreenTree::Empty => false,
            GreenTree::None => true,
        }
    }

    /// Returns the current cursor if it's not empty, otherwise returns false.
    pub fn is_empty(&self) -> bool {
        match self.value.borrow().green() {
            GreenTree::Leaf { data, .. } => !data.children.is_empty(),
            GreenTree::Token(..) => false,
            GreenTree::Empty => false,
            GreenTree::None => false,
        }
    }

    /// Creates a new cursor with the given value.
    pub fn of(value: T) -> Self {
        Self {
            value: Rc::new(RefCell::new(Value::Ref(Rc::new(value)))),
            _marker: PhantomData,
        }
    }

    /// Returns the current cursor if it's not empty, otherwise returns a default value.
    pub fn as_leaf(&self) -> Rc<T> {
        match &*self.value.borrow() {
            Value::Green(green) => Rc::new(T::new(green.clone())),
            Value::Ref(value) => value.clone(),
        }
    }
}

impl<T: Leaf> From<Rc<T>> for Cursor<T> {
    fn from(value: Rc<T>) -> Self {
        Self {
            value: Rc::new(RefCell::new(Value::Ref(value))),
            _marker: PhantomData,
        }
    }
}

impl<T: Leaf> Default for Cursor<T> {
    fn default() -> Self {
        Self {
            value: Rc::new(RefCell::new(Value::Green(Default::default()))),
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
            GreenTree::Empty => vec![],
            GreenTree::None => vec![],
            GreenTree::Token(..) => vec![],
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
        }
    }

    fn unwrap(self) -> GreenTree {
        let children = self
            .into_iter()
            .map(|x| x.unwrap().as_child())
            .collect::<Vec<_>>();

        let tree = Tree {
            name: None,
            kind: TreeKind::ListTree,
            children,
        };

        GreenTree::Leaf {
            data: Spanned::new(Loc::default(), tree),
            names: Rc::default(),
            children: Default::default(),
            synthetic: false,
        }
    }
}

impl<T: Node + Leaf> From<Option<T>> for Cursor<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => Self::of(value),
            None => Self {
                value: Rc::new(RefCell::new(Value::Green(GreenTree::None))),
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
    type Output = Rc<T>;

    type Residual = Option<std::convert::Infallible>;

    fn from_output(output: Self::Output) -> Self {
        Self::from(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match &*self.value.borrow() {
            Value::Green(leaf @ GreenTree::Leaf { .. }) => {
                ControlFlow::Continue(Rc::new(T::new(leaf.clone())))
            }
            Value::Green(GreenTree::Token(lexeme)) => match lexeme.downcast_ref::<T>() {
                Some(value) => ControlFlow::Continue(Rc::new(value.clone())),
                None => ControlFlow::Break(None),
            },
            Value::Green(GreenTree::Empty) => ControlFlow::Break(None),
            Value::Green(GreenTree::None) => ControlFlow::Break(None),
            Value::Ref(value) => ControlFlow::Continue(value.clone()),
        }
    }
}
