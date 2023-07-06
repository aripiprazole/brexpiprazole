use std::sync::{Arc, RwLock};
use std::{any::Any, borrow::Cow, collections::HashMap, rc::Rc};

use asena_interner::Intern;
use asena_span::Spanned;

use crate::node::{Child, Named, Tree, TreeKind};
use crate::token::token_set::HasTokens;

use super::*;

#[derive(Debug, Clone)]
pub struct AstLeaf {
    pub(crate) data: Intern<Spanned<Tree>>,

    synthetic: bool,

    children: HashMap<LeafKey, Child>,

    keys: Arc<RwLock<HashMap<&'static str, Rc<dyn Any>>>>,

    /// Lazy names' hash map, they have to exist, to make the tree mutable.
    ///
    /// E.g: I can't set the `lhs` node for `binary` tree, if the tree is immutable, so the
    /// lazy names should be used to compute that things.
    /// ```rs
    /// binary.lhs()
    /// ```
    names: Arc<RwLock<HashMap<LeafKey, Arc<dyn Any>>>>,
}

/// A wrapper for the [Tree] to make it mutable and have mutable named children.
///
/// It is used to traverse the tree, and to modify it, and can be an [GreenTree::Empty] node,
/// that is used to mark the tree as invalid, and not fail the compiler.
#[derive(Clone)]
pub enum GreenTree {
    Leaf(AstLeaf),
    Vec(Vec<GreenTree>),
    Token(Lexeme<Rc<dyn Any>>),

    /// A node that is supposed to be None.
    None,

    /// An empty node, that is used to mark the tree as invalid, and not fail the compiler.
    Empty,
}

impl GreenTree {
    pub fn new(data: Intern<Spanned<Tree>>) -> Self {
        Self::Leaf(AstLeaf {
            children: children(&data),
            data,
            synthetic: false,
            names: AstLeaf::new_ref(HashMap::new()),
            keys: AstLeaf::new_ref(HashMap::new()),
        })
    }

    pub fn of(kind: TreeKind) -> Self {
        let mut data: Intern<Spanned<Tree>> = Default::default();
        data.value.kind = kind;

        Self::Leaf(AstLeaf {
            data,
            children: HashMap::default(),
            synthetic: true,
            names: AstLeaf::new_ref(HashMap::new()),
            keys: AstLeaf::new_ref(HashMap::new()),
        })
    }

    pub fn as_node<T>(&self) -> T
    where
        T: Leaf,
    {
        Leaf::make(self.clone()).unwrap_or_default()
    }

    /// Checks if the tree matches the given kind.
    pub fn matches(&self, nth: usize, kind: TokenKind) -> bool {
        match self {
            Self::Leaf(leaf) => leaf.data.matches(nth, kind),
            _ => false,
        }
    }

    /// Returns the [Spanned] value of the tree, if it's not an error node, then it should
    /// return the default value.
    pub fn or_empty(self) -> Intern<Spanned<Tree>> {
        match self {
            Self::Leaf(leaf) => leaf.data,
            _ => Default::default(),
        }
    }

    /// Returns the [TreeKind] of the tree, and if it's not a [AstLeaf], it will return an
    /// Error kind.
    pub fn kind(&self) -> TreeKind {
        match self {
            Self::Leaf(leaf) => leaf.data.kind,
            _ => TreeKind::Error,
        }
    }

    /// Returns the value of the given name, if it exists, otherwise it will return the default
    /// value.
    pub fn spanned(&self) -> Spanned<()> {
        match self {
            GreenTree::Leaf(leaf) => leaf.data.replace(()),
            GreenTree::Token(lexeme) => lexeme.token.replace(()),
            _ => Spanned::default(),
        }
    }

    /// Returns if the value is the only element in the tree.
    pub fn is_single(&self) -> bool {
        match self {
            Self::Leaf(leaf) => leaf.data.is_single(),
            Self::Token(..) => true,
            _ => false,
        }
    }

    /// Returns the tree children, if it's not an error node.
    pub fn children(&mut self) -> Option<&mut Vec<Child>> {
        match self {
            Self::Leaf(leaf) => Some(&mut leaf.data.children),
            _ => None,
        }
    }

    /// Returns filtered cursor to the children, if it's not an error node.
    pub fn filter<T: Leaf + Node>(&self) -> Cursor<Vec<T>> {
        match self {
            Self::Leaf(leaf) => leaf.data.filter(),
            _ => Cursor::empty(),
        }
    }

    /// Returns a terminal node, if it's not an error node.
    pub fn any_token(&self, kind: TokenKind) -> Vec<Intern<Spanned<Token>>> {
        match self {
            Self::Leaf(leaf) => leaf.data.token(kind),
            _ => vec![],
        }
    }

    pub fn token(&self, kind: TokenKind) -> Intern<Spanned<Token>> {
        match self {
            Self::Leaf(leaf) => leaf.data.token(kind).first().cloned().unwrap_or_default(),
            _ => Default::default(),
        }
    }

    /// Returns a terminal node, if it's not an error node.
    pub fn terminal<T: Terminal + 'static>(&self, nth: usize) -> Cursor<Lexeme<T>> {
        match self {
            Self::Leaf(leaf) => leaf.data.terminal(nth),
            _ => Cursor::empty(),
        }
    }

    /// Returns terminal filtered cursor to the children, if it's not an error node.
    pub fn filter_terminal<T: Terminal + 'static>(&self) -> Cursor<Vec<Lexeme<T>>> {
        match self {
            Self::Leaf(leaf) => leaf.data.filter_terminal(),
            _ => Cursor::empty(),
        }
    }

    /// Returns a leaf node, if it's not an error node.
    pub fn at<T: Node + Leaf>(&self, nth: usize) -> Cursor<T> {
        match self {
            Self::Leaf(leaf) => leaf.data.at(nth),
            _ => Cursor::empty(),
        }
    }

    /// Returns if the tree has the given name in the current name hash map.
    pub fn has(&self, name: LeafKey) -> bool {
        match self {
            Self::Leaf(leaf) => matches!(leaf.children.get(name), Some(..)),
            _ => false,
        }
    }

    /// Returns a cursor to the named child, if it's not an error node.
    pub fn named_at<A: Leaf + Node + 'static>(&self, name: LeafKey) -> Cursor<A> {
        match self {
            Self::Leaf(leaf) => {
                let borrow = leaf.names();
                let Some(child) = borrow.get(name).and_then(|x| x.downcast_ref::<Cursor<A>>()) else {
                    return match leaf.children.get(name) {
                        Some(Child::Token(..)) => Cursor::empty(),
                        Some(Child::Tree(ref tree)) => {
                            A::make(GreenTree::new(tree.clone())).into()
                        },
                        None => Cursor::empty(),
                    };
                };

                child.clone()
            }
            _ => Cursor::empty(),
        }
    }

    /// Returns a cursor to the named terminal, if it's not an error node.
    pub fn named_terminal<A: Terminal + 'static>(&self, name: LeafKey) -> Cursor<Lexeme<A>> {
        match self {
            Self::Leaf(leaf) => {
                let names = leaf.names();
                let Some(child) = names.get(name).and_then(|x| x.downcast_ref::<Cursor<Lexeme<A>>>()) else {
                    return match leaf.children.get(name) {
                        Some(Child::Tree(..)) => Cursor::empty(),
                        Some(Child::Token(ref token)) => {
                            Lexeme::<A>::terminal(token.clone()).into()
                        },
                        None => Cursor::empty(),
                    };
                };

                child.clone()
            }
            _ => Cursor::empty(),
        }
    }

    /// Creates a new node from the current node, if it's a leaf node, it will reset the names, and
    /// keys hash maps, and it will compute the named children again, to really duplicate the node,
    /// use [GreenTree::clone].
    ///
    /// This method is useful to create a new node from a leaf node, and then insert it into the
    /// tree.
    pub fn as_new_node(&self) -> Self {
        match self {
            Self::Leaf(leaf) => Self::Leaf(AstLeaf {
                data: leaf.data.clone(),
                synthetic: leaf.synthetic,
                children: children(&leaf.data),
                names: AstLeaf::new_ref(HashMap::new()),
                keys: AstLeaf::new_ref(HashMap::new()),
            }),
            _ => self.clone(),
        }
    }

    /// Inserts a key into the tree, and returns the value. It's not the same of [GreenTree::insert]
    /// because, [GreenTree::insert] sets in the `names` field
    pub fn insert_key<T: Key>(&self, key: T, value: T::Value) -> Rc<T::Value> {
        if let Self::Leaf(leaf) = self {
            leaf.keys_mut()
                .insert(key.name(), Rc::new(value))
                .unwrap()
                .downcast::<T::Value>()
                .unwrap()
        } else {
            Rc::new(value)
        }
    }

    /// Returns the value of the key, if it exists, otherwise it will return the default value.
    pub fn key<T: Key>(&self, key: T) -> Rc<T::Value> {
        let value = T::Value::default();
        if let Self::Leaf(leaf) = self {
            if let Some(value) = leaf.keys().get(key.name()) {
                return value.clone().downcast::<T::Value>().unwrap();
            }

            leaf.keys_mut()
                .insert(key.name(), Rc::new(value))
                .unwrap()
                .downcast::<T::Value>()
                .unwrap()
        } else {
            Rc::new(value)
        }
    }

    pub fn insert<T: 'static>(&self, name: LeafKey, value: T)
    where
        T: Node + Leaf,
    {
        if let Self::Leaf(leaf) = self {
            leaf.names_mut().insert(name, Arc::new(Cursor::of(value)));
        }
    }

    /// Memoizes the value of the given function, and returns a new [Cursor] instance, and
    /// if the value is already memoized, it will return the memoized value.
    ///
    /// This function is used to memoize the values of the named children, to make the tree
    /// mutable.
    pub fn memoize<F, T: Leaf + Clone + 'static>(&self, name: &'static str, f: F) -> Cursor<T>
    where
        F: Fn(&Self) -> Cursor<T>,
        T: Node,
    {
        let tree @ Self::Leaf(leaf) = self else {
            return Cursor::empty();
        };

        if let Some(x) = leaf.names().get(name) {
            return x.downcast_ref::<Cursor<T>>().unwrap().clone();
        }

        let cursor = f(tree);
        leaf.names_mut().insert(name, Arc::new(cursor.clone()));
        cursor
    }
}

impl Default for GreenTree {
    fn default() -> Self {
        Self::Leaf(AstLeaf {
            data: Default::default(),
            children: HashMap::new(),
            synthetic: false,
            keys: AstLeaf::new_ref(HashMap::new()),
            names: AstLeaf::new_ref(HashMap::new()),
        })
    }
}

impl Debug for GreenTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Token(lexeme) => f
                .debug_struct("Token")
                .field("kind", &lexeme.token.kind.name())
                .field("value", lexeme)
                .finish(),
            Self::Vec(children) => f.debug_tuple("Vec").field(children).finish(),
            Self::Leaf(leaf) => write!(f, "Leaf({:#?})", leaf.data),
            Self::Empty => write!(f, "Empty"),
            Self::None => write!(f, "None"),
        }
    }
}

impl From<Intern<Spanned<Tree>>> for GreenTree {
    fn from(value: Intern<Spanned<Tree>>) -> Self {
        Self::new(value)
    }
}

impl HasTokens for GreenTree {
    fn tokens(&self) -> Vec<Intern<Spanned<Token>>> {
        match self {
            Self::Leaf(leaf) => leaf.data.tokens(),
            Self::Vec(vec) => vec.iter().flat_map(|tree| tree.tokens()).collect(),
            Self::Token(lexeme) => vec![lexeme.token.clone()],
            Self::None => vec![],
            Self::Empty => vec![],
        }
    }
}

impl Located for GreenTree {
    fn location(&self) -> Cow<'_, Loc> {
        match self {
            Self::Leaf(leaf) => Cow::Borrowed(&leaf.data.span),
            Self::Token(ref lexeme) => Cow::Borrowed(&lexeme.token.span),
            _ => Cow::Owned(Loc::Synthetic),
        }
    }
}

/// Computes the named children of the given tree, and returns a hash map with the named children.
///
/// This function is used to compute the tree that the `name` property is not [None].
fn children(data: &Intern<Spanned<Tree>>) -> HashMap<LeafKey, Child> {
    let mut named_children = HashMap::new();

    for child in &data.children {
        match child {
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

impl AstLeaf {
    fn new_ref<T>(value: T) -> Arc<RwLock<T>> {
        Arc::new(RwLock::new(value))
    }

    fn names(&self) -> std::sync::RwLockReadGuard<'_, HashMap<&str, Arc<dyn Any>>> {
        self.names.read().unwrap()
    }

    fn names_mut(&self) -> std::sync::RwLockWriteGuard<'_, HashMap<&'static str, Arc<dyn Any>>> {
        self.names.write().unwrap()
    }

    fn keys(&self) -> std::sync::RwLockReadGuard<'_, HashMap<&str, Rc<dyn Any>>> {
        self.keys.read().unwrap()
    }

    fn keys_mut(&self) -> std::sync::RwLockWriteGuard<'_, HashMap<&'static str, Rc<dyn Any>>> {
        self.keys.write().unwrap()
    }
}
