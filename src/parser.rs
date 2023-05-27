use std::iter::Peekable;

use crate::ast::{App, Binary, Expr, ExprRef, FunctionId, GlobalId, Literal};
use crate::lexer::Token;
use crate::span::Spanned;

pub type TokenRef = Spanned<Token>;

pub type StringRef = Spanned<String>;

#[derive(Debug, Clone, Copy)]
pub enum ParseError {
    UnexpectedToken,
    CantParsePrimary,
}

pub type Result<T, E = Spanned<ParseError>> = std::result::Result<T, E>;

/// The language parser struct, it takes a [Token] iterator, that can be lazy or eager initialized
/// to advance and identify tokens on the programming language.
pub struct Parser<'a, S: Iterator<Item = Spanned<Token>>> {
    pub source: &'a str,
    pub index: usize,
    pub stream: Peekable<S>,
}

impl<'a, S: Iterator<Item = Spanned<Token>>> Parser<'a, S> {
    pub fn new(source: &'a str, stream: Peekable<S>) -> Self {
        Self {
            index: 0,
            source,
            stream,
        }
    }

    //>>>Parser functions
    pub fn expr(&mut self) -> Result<ExprRef> {
        self.binary()
    }

    pub fn binary(&mut self) -> Result<ExprRef> {
        let mut lhs = self.app()?;

        loop {
            let next = self.next();

            let Token::Symbol(symbol) = next.value() else {
                break;
            };

            let fn_id = Spanned::new(next.span().clone(), FunctionId::new(symbol));
            let rhs = self.app()?;

            // Combines two locations
            let span = lhs.span.start..rhs.span.end;

            lhs = ExprRef::new(span, Expr::Binary(Binary { lhs, fn_id, rhs }))
        }

        Ok(lhs)
    }

    pub fn app(&mut self) -> Result<ExprRef> {
        let mut callee = self.primary()?;

        while let Some(argument) = self.catch(Parser::primary)? {
            // Combines two locations
            let span = callee.span.start..argument.span.end;

            callee = ExprRef::new(span, Expr::App(App { callee, argument }))
        }

        Ok(callee)
    }

    pub fn primary(&mut self) -> Result<ExprRef> {
        use Token::*;

        let current = self.peek();
        let value = match current.value() {
            // Booleans
            True => Expr::Literal(Literal::True),
            False => Expr::Literal(Literal::False),

            // Integers
            Int8(n, signed) => Expr::Literal(Literal::Int8(*n, *signed)),
            Int16(n, signed) => Expr::Literal(Literal::Int16(*n, *signed)),
            Int32(n, signed) => Expr::Literal(Literal::Int32(*n, *signed)),
            Int64(n, signed) => Expr::Literal(Literal::Int64(*n, *signed)),
            Int128(n, signed) => Expr::Literal(Literal::Int128(*n, *signed)),

            // Floating pointers
            Float32(n) => Expr::Literal(Literal::Float32(*n)),
            Float64(n) => Expr::Literal(Literal::Float64(*n)),

            // String
            String(content) => {
                // Remove the `"` tokens of the string, they start with 1 gap in the start and in
                // the end of the content.
                let content = content[1..(content.len() - 1)].to_string();

                Expr::Literal(Literal::String(content))
            }

            // Starts with a Global expression, and its needed to be resolved in a further step, it
            // can be either a [Global] or a [Local].
            Ident(..) => {
                // skip <identifier>
                //
                // It does not uses the Ident(..) pattern, because of the location, we need locality
                // of the ast.
                let ident = self.identifier()?.map(|s| FunctionId::new(&s));

                // Creates a new path.
                let mut path = vec![ident];
                while let Token::Dot = self.peek().value() {
                    self.next(); // skip `.`
                    let fn_id = self.identifier()?.map(|s| FunctionId::new(&s));
                    path.push(fn_id); // adds new `.` <identifier>
                }

                // Creates a new location combining the first, and the last points in the source code
                let a = path.first().unwrap().span();
                let b = path.last().map(Spanned::span).unwrap_or(a);

                return Ok(ExprRef::new(a.start..b.end, Expr::Global(GlobalId(path))));
            }

            //>>>Composed tokens
            // Group expression
            LeftParen => {
                self.next(); // skip '('
                let expr = self.expr()?;
                self.expect(Token::RightParen)?; // consumes ')'

                return Ok(current.swap(Expr::Group(expr)));
            }

            // Help expression
            Help => {
                self.next(); // skip '?'
                let expr = self.expr()?;

                return Ok(current.swap(Expr::Help(expr)));
            }
            _ => return self.end_diagnostic(ParseError::CantParsePrimary),
        };

        self.next(); // Skips if hadn't any error

        Ok(current.swap(value))
    }

    fn identifier(&mut self) -> Result<StringRef> {
        self.eat(|next| match next.value() {
            Token::Ident(content) => Some(next.replace(content.clone())),

            // Accepts symbol, so the parser is able to parse something like `Functor.<$>`
            Token::Symbol(content) => Some(next.replace(content.clone())),
            _ => None,
        })
    }

    fn expect(&mut self, token: Token) -> Result<TokenRef> {
        self.eat(|next| {
            if next.value() == &token {
                Some(next.clone())
            } else {
                None
            }
        })
    }

    fn catch<T, F>(&mut self, mut f: F) -> Result<Option<T>>
    where
        F: FnMut(&mut Self) -> Result<T>,
    {
        let current_index = self.index;

        match f(self) {
            Ok(value) => Ok(Some(value)),
            Err(..) if self.index == current_index => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn eat<T, F>(&mut self, f: F) -> Result<T>
    where
        F: Fn(&TokenRef) -> Option<T>,
    {
        let next = self.peek();
        match f(&next) {
            Some(value) => {
                self.next();
                Ok(value)
            }
            None => Err(next.swap(ParseError::UnexpectedToken)),
        }
    }

    fn next(&mut self) -> TokenRef {
        self.index += 1;

        self.stream.next().unwrap()
    }

    fn end_diagnostic<T>(&mut self, error: ParseError) -> Result<T, Spanned<ParseError>> {
        Err(self.stream.peek().unwrap().replace(error))
    }

    fn peek(&mut self) -> Spanned<Token> {
        self.stream.peek().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;

    use super::*;

    #[test]
    fn it_works() {
        let code = "Nat.+ 10 10";

        let stream = Lexer::new(code);
        let mut parser = Parser::new(code, stream.peekable());

        println!("{:#?}", parser.expr().unwrap())
    }
}
