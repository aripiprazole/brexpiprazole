use std::rc::Rc;

use super::*;

pub struct Cursor<T> {
    pub(crate) value: Arc<RefCell<Value<T>>>,

    /// Children marked with name, to be accessed fast.
    pub(crate) children: HashMap<LeafKey, Spanned<Child>>,
}

impl<T: Leaf> Cursor<T> {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn set(&self, value: Cursor<T>) {
        self.value.replace(value.value.borrow().clone());
    }

    pub fn of(value: T) -> Self {
        Self {
            value: Arc::new(RefCell::new(Value::Value(Rc::new(value)))),
            children: Default::default(),
        }
    }

    pub fn from_rc(value: Rc<T>) -> Self {
        Self {
            value: Arc::new(RefCell::new(Value::Value(value))),
            children: Default::default(),
        }
    }

    pub fn new<I: Into<GreenTree>>(value: I) -> Self {
        let tree: GreenTree = value.into();
        let children = compute_named_children(&tree);

        Self {
            value: Arc::new(RefCell::new(Value::Ref(tree))),
            children,
        }
    }

    pub fn as_new_node(&self) -> Self
    where
        T: Clone,
    {
        let new_value = self.value.borrow().clone();
        let children = match new_value {
            Value::Ref(ref tree) => compute_named_children(tree),
            Value::Value(..) => Default::default(),
        };

        Self {
            value: Arc::new(RefCell::new(new_value)),
            children,
        }
    }

    pub fn try_as_leaf(&self) -> Option<Rc<T>>
    where
        T: Clone,
    {
        match &*self.value.borrow() {
            Value::Ref(GreenTree::Leaf { data, .. }) => T::make(data.clone()).map(Rc::new),
            Value::Ref(GreenTree::Error) => None,
            Value::Value(value) => Some(value.clone()),
        }
    }

    pub fn as_leaf(&self) -> Rc<T>
    where
        T: Clone + Default,
    {
        self.try_as_leaf().unwrap_or_default()
    }

    pub fn is_empty(&self) -> bool {
        match &*self.value.borrow() {
            Value::Ref(GreenTree::Leaf { .. }) => true,
            Value::Ref(GreenTree::Error) => false,
            Value::Value(..) => true,
        }
    }
}

pub enum CursorCow<'a, T> {
    Owned(T),
    Borrowed(&'a T),
}

impl<T: Leaf> Cursor<Vec<T>> {
    pub fn first(self) -> Cursor<T> {
        self.as_leaf().first().cloned().into()
    }
}

impl<T: Leaf> From<Vec<T>> for Cursor<Vec<T>> {
    fn from(value: Vec<T>) -> Self {
        Cursor::of(value)
    }
}

impl<T: Leaf> From<Option<T>> for Cursor<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => Self::of(value),
            None => Self::empty(),
        }
    }
}

impl<T: Leaf + Display + Default> Display for Cursor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_leaf())
    }
}

impl<T: Leaf + Debug + Default> Debug for Cursor<T> {
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
            children: self.children.clone(),
        }
    }
}

impl<T: Leaf> FromResidual for Cursor<T> {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        match residual {
            Some(_) => unreachable!(),
            None => Cursor::empty(),
        }
    }
}

impl<T: Leaf> Try for Cursor<T> {
    type Output = Rc<T>;

    type Residual = Option<std::convert::Infallible>;

    fn from_output(output: Self::Output) -> Self {
        Self::from_rc(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match &*self.value.borrow() {
            Value::Ref(GreenTree::Leaf { data, .. }) => match T::make(data.clone()) {
                Some(value) => ControlFlow::Continue(Rc::new(value)),
                None => ControlFlow::Break(None),
            },
            Value::Ref(GreenTree::Error) => ControlFlow::Break(None),
            Value::Value(value) => ControlFlow::Continue(value.clone()),
        }
    }
}

fn compute_named_children(tree: &GreenTree) -> HashMap<LeafKey, Spanned<Child>> {
    let GreenTree::Leaf { data, .. } = tree else {
        return HashMap::new();
    };

    let mut named_children = HashMap::new();

    for child in &data.children {
        match child.value() {
            Child::Tree(tree) => {
                if let Some(name) = tree.name {
                    named_children.insert(name, child.clone());
                }
            }
            Child::Token(token) => {
                if let Some(name) = token.name {
                    named_children.insert(name, child.clone());
                }
            }
        }
    }

    named_children
}
