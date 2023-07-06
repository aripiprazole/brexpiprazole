use asena_span::Spanned;

use crate::token::token_set::HasTokens;
use crate::token::Token;

use super::*;

/// Represents a lexeme, a token with a value, represented in the Rust language.
#[derive(Clone)]
pub struct Lexeme<T> {
    pub token: Intern<Spanned<Token>>,
    pub value: T,

    /// If the lexeme is `None`, it means that the lexeme is a placeholder.
    pub(crate) is_none: bool,
}

impl<T> Lexeme<T> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Lexeme<U> {
        Lexeme {
            token: self.token,
            value: f(self.value),
            is_none: false,
        }
    }

    /// Maps the token and the value of the lexeme.
    ///
    /// # Example
    /// ```rust,norun
    /// use asena_span::{Loc, Spanned};
    /// use asena_ast::token::{Token, TokenKind};
    /// use asena_ast::ast::Lexeme;
    ///
    /// let lexeme = Lexeme::<String> {
    ///    token: Spanned::new(Loc::default(), Token::new(TokenKind::Error, "")),
    ///    value: "hello".to_string(),
    /// };
    ///
    /// let lexeme = lexeme.map_token(|s, t| {
    ///    format!("{}: {:?}", s, t)
    /// });
    /// ```
    pub fn map_token<U>(self, f: impl FnOnce(T, &Spanned<Token>) -> U) -> Lexeme<U> {
        let value = f(self.value, &self.token);
        Lexeme {
            token: self.token,
            is_none: false,
            value,
        }
    }
}

impl<T: Default> Default for Lexeme<T> {
    fn default() -> Self {
        Self {
            token: Default::default(),
            value: Default::default(),
            is_none: false,
        }
    }
}

pub trait LexemeWalkable: Sized {
    type Walker<'a>;

    fn lexeme_walk(value: Lexeme<Self>, walker: &mut Self::Walker<'_>);
}

pub trait LexemeListenable: Sized {
    type Listener<'a>;

    fn lexeme_listen(value: Lexeme<Self>, walker: &mut Self::Listener<'_>);
}

impl<T: Walkable> LexemeWalkable for Option<T> {
    fn lexeme_walk(value: Lexeme<Self>, walker: &mut Self::Walker<'_>) {
        if let Some(value) = value.value {
            value.walk(walker);
        }
    }

    type Walker<'a> = T::Walker<'a>;
}

impl<T: LexemeListenable + Clone> Listenable for Lexeme<T> {
    type Listener<'a> = T::Listener<'a>;

    fn listen(&self, listener: &mut Self::Listener<'_>) {
        T::lexeme_listen(self.clone(), listener)
    }
}

impl<T: LexemeWalkable + Clone> Walkable for Lexeme<T> {
    type Walker<'a> = T::Walker<'a>;

    fn walk(&self, walker: &mut Self::Walker<'_>) {
        T::lexeme_walk(self.clone(), walker)
    }
}

impl<T: Node> Node for Option<T> {
    fn new<I: Into<GreenTree>>(tree: I) -> Self {
        let tree: GreenTree = tree.into();

        match tree {
            GreenTree::None => None,
            GreenTree::Empty => None,
            _ => Some(T::new(tree)),
        }
    }

    fn unwrap(self) -> GreenTree {
        match self {
            Some(vale) => vale.unwrap(),
            None => GreenTree::None,
        }
    }
}

impl<T> HasTokens for Lexeme<T> {
    fn tokens(&self) -> Vec<Intern<Spanned<Token>>> {
        self.token.tokens()
    }
}

impl<T> Located for Lexeme<T> {
    fn location(&self) -> Cow<'_, Loc> {
        Cow::Borrowed(&self.token.span)
    }
}

impl<T: std::fmt::Display> std::fmt::Display for Lexeme<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_none {
            write!(f, "None[{}]", std::any::type_name::<T>())
        } else {
            std::fmt::Display::fmt(&self.value, f)
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Lexeme<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_none {
            write!(f, "None[{}]", std::any::type_name::<T>())
        } else {
            std::fmt::Debug::fmt(&self.value, f)
        }
    }
}

impl<T> std::ops::Deref for Lexeme<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> std::borrow::Borrow<T> for Lexeme<T> {
    fn borrow(&self) -> &T {
        &self.value
    }
}

impl<T: Terminal + 'static> Leaf for Lexeme<T> {
    fn terminal(token: Intern<Spanned<Token>>) -> Option<Self> {
        let spanned = token.clone();
        let terminal = <T as Terminal>::terminal(token)?;

        Some(Self {
            token: spanned,
            value: terminal,
            is_none: false,
        })
    }
}

impl<T: Leaf + 'static> Node for Lexeme<T> {
    fn new<I: Into<GreenTree>>(tree: I) -> Self {
        match tree.into() {
            ref tree @ GreenTree::Leaf(ref leaf) => {
                let token = if leaf.data.children.is_empty() {
                    #[cfg(debug_assertions)]
                    println!("Lexeme::new: Leaf node has no children: {}", leaf.data.kind);
                    Default::default()
                } else {
                    let first_item = leaf.data.single().clone();
                    let spanned = leaf.data.replace(first_item);
                    Intern::new(spanned)
                };

                Self {
                    token,
                    value: T::make(tree.clone()).unwrap_or_default(),
                    is_none: false,
                }
            }
            GreenTree::Token(lexeme) => {
                let value = match lexeme.value.downcast_ref::<T>() {
                    Some(value) => value.clone(),
                    None => return Default::default(),
                };

                Self {
                    token: lexeme.token,
                    is_none: false,
                    value,
                }
            }
            GreenTree::None => Self {
                token: Default::default(),
                value: T::default(),
                is_none: true,
            },
            _ => Self::default(),
        }
    }

    fn unwrap(self) -> GreenTree {
        GreenTree::Token(Lexeme {
            token: self.token,
            value: Rc::new(self.value),
            is_none: self.is_none,
        })
    }
}
