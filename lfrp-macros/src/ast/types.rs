use super::path;

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Bracket, Paren, Underscore};
use syn::{bracketed, parenthesized};
use syn::{Ident, Result, Token};

#[derive(Debug)]
pub struct TypeParen {
    paren_token: Paren,
    ty: Box<Type>,
}

#[derive(Debug)]
pub struct TypeList {
    bracket_token: Bracket,
    ty: Box<Type>,
}

#[derive(Debug)]
pub struct TypeTuple {
    paren_token: Paren,
    elems: Punctuated<Type, Token![,]>,
}

#[derive(Debug)]
pub struct TypeInfer {
    underscore_token: Underscore,
}

#[derive(Debug)]
pub struct TypePath {
    path: path::Path,
}

impl Parse for TypePath {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(TypePath {
            path: input.parse()?,
        })
    }
}

#[derive(Debug)]
pub enum Type {
    List(TypeList),
    Tuple(TypeTuple),
    Paren(TypeParen),
    Infer(TypeInfer),
    Path(TypePath),
}

impl Parse for Type {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Ident) {
            Ok(input.parse().map(Type::Path)?)
        } else if lookahead.peek(Bracket) {
            let content;
            Ok(Type::List(TypeList {
                bracket_token: bracketed!(content in input),
                ty: content.parse()?,
            }))
        } else if lookahead.peek(Paren) {
            let content;
            let paren_token = parenthesized!(content in input);

            if content.is_empty() {
                return Ok(Type::Tuple(TypeTuple {
                    paren_token,
                    elems: Punctuated::new(),
                }));
            }

            let first: Type = content.parse()?;
            if content.peek(Token![,]) {
                let mut elems = Punctuated::new();
                elems.push_value(first);
                elems.push_punct(content.parse()?);
                let rest: Punctuated<Type, Token![,]> = content.parse_terminated(Parse::parse)?;
                elems.extend(rest);

                Ok(Type::Tuple(TypeTuple { paren_token, elems }))
            } else {
                Ok(Type::Paren(TypeParen {
                    paren_token,
                    ty: Box::new(first),
                }))
            }
        } else if lookahead.peek(Token![_]) {
            Ok(Type::Infer(TypeInfer {
                underscore_token: input.parse()?,
            }))
        } else {
            Err(lookahead.error())
        }
    }
}
