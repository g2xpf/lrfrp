use super::path;

use std::borrow::Borrow;
use std::fmt;

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Bracket, Paren, Underscore};
use syn::{bracketed, parenthesized};
use syn::{Ident, Result, Token};

use quote::{quote, ToTokens};

use proc_macro2::TokenStream;

#[derive(Clone, Debug)]
pub enum Type {
    List(TypeList),
    Tuple(TypeTuple),
    Paren(TypeParen),
    Infer(TypeInfer),
    Path(TypePath),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Type::*;
        match self {
            List(ref ty) => ty.fmt(f),
            Tuple(ref ty) => ty.fmt(f),
            Paren(ref ty) => ty.fmt(f),
            Infer(ref ty) => ty.fmt(f),
            Path(ref ty) => ty.fmt(f),
        }
    }
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

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Type::*;

        match self {
            List(t) => t.to_tokens(tokens),
            Tuple(t) => t.to_tokens(tokens),
            Paren(t) => t.to_tokens(tokens),
            Infer(t) => t.to_tokens(tokens),
            Path(t) => t.to_tokens(tokens),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TypeParen {
    paren_token: Paren,
    ty: Box<Type>,
}

impl fmt::Display for TypeParen {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.ty)
    }
}

impl ToTokens for TypeParen {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty = &self.ty;
        let type_paren = quote! {
            (#ty)
        };
        tokens.extend(type_paren);
    }
}

#[derive(Clone, Debug)]
pub struct TypeList {
    bracket_token: Bracket,
    ty: Box<Type>,
}

impl fmt::Display for TypeList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.ty)
    }
}

impl ToTokens for TypeList {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty = &self.ty;
        let type_list = quote! {
            [#ty]
        };
        tokens.extend(type_list);
    }
}

#[derive(Clone, Debug)]
pub struct TypeTuple {
    paren_token: Paren,
    elems: Punctuated<Type, Token![,]>,
}

impl fmt::Display for TypeTuple {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:?})", self.elems)
    }
}

impl ToTokens for TypeTuple {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let elems = &self.elems;
        let type_tuple = quote! {
            (#elems)
        };
        tokens.extend(type_tuple);
    }
}

#[derive(Clone, Debug)]
pub struct TypeInfer {
    underscore_token: Underscore,
}

impl fmt::Display for TypeInfer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "_")
    }
}

impl ToTokens for TypeInfer {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let underscore_token = &self.underscore_token;
        let type_infer = quote! {
            #underscore_token
        };
        tokens.extend(type_infer);
    }
}

#[derive(Clone, Debug)]
pub struct TypePath {
    path: path::Path,
}

impl fmt::Display for TypePath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ident: &Ident = self.path.borrow();
        write!(f, "{}", ident.to_string())
    }
}

impl Parse for TypePath {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(TypePath {
            path: input.parse()?,
        })
    }
}

impl ToTokens for TypePath {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.path.to_tokens(tokens)
    }
}
