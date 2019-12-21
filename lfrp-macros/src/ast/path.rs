use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::Ident;
use syn::Result;
use syn::Token;

use crate::lfrp_ir::types;

use super::types::Type;

use quote::{quote, ToTokens};

use proc_macro2::TokenStream;

use std::borrow::Borrow;
use std::ptr;

#[derive(Clone, Debug)]
pub enum Path {
    Segment(PathSegment),
    TypedSegment(PathSegment, types::Type),
}

impl Path {
    pub fn typing(&mut self, ty: &types::Type) {
        use Path::*;
        let ty = ty.clone();
        unsafe {
            ptr::write(
                self,
                match ptr::read(self) {
                    Segment(s) => TypedSegment(s, ty),
                    TypedSegment(s, _) => TypedSegment(s, ty),
                },
            );
        }
    }
}

impl Borrow<Ident> for Path {
    fn borrow(&self) -> &Ident {
        use Path::*;
        match self {
            Segment(ref segment) => segment.borrow(),
            TypedSegment(segment, _) => segment.borrow(),
        }
    }
}

impl<T> From<T> for Path
where
    T: Into<PathSegment>,
{
    fn from(t: T) -> Self {
        Path::Segment(t.into())
    }
}

impl Parse for Path {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Path::Segment(input.parse()?))
    }
}

impl ToTokens for Path {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Path::*;
        match self {
            Segment(segment) => segment.to_tokens(tokens),
            TypedSegment(segment, ty) => {
                let ident: &Ident = segment.borrow();
                match ty {
                    types::Type::Mono(types::TypeMono::Type(_)) => ident.to_tokens(tokens),
                    types::Type::Lifted(types::TypeLifted::Cell(_)) => {
                        tokens.extend(quote! {self.cell.#ident})
                    }
                    types::Type::Mono(types::TypeMono::Args(_)) => {
                        tokens.extend(quote! { self.args.#ident })
                    }
                    types::Type::Lifted(types::TypeLifted::Signal(ref ty)) => match ty {
                        types::TypeSignal::Local(_) => ident.to_tokens(tokens),
                        types::TypeSignal::Input(_) => tokens.extend(quote! { input.#ident }),
                        types::TypeSignal::Output(_) => {
                            tokens.extend(quote! { self.output.#ident })
                        }
                    },
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct PathSegment {
    pub ident: Ident,
    pub arguments: PathArguments,
}

impl Borrow<Ident> for PathSegment {
    fn borrow(&self) -> &Ident {
        &self.ident
    }
}

impl From<Ident> for PathSegment {
    fn from(ident: Ident) -> Self {
        PathSegment {
            ident,
            arguments: PathArguments::None,
        }
    }
}

impl Parse for PathSegment {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse()?;
        let lookahead = input.lookahead1();

        if lookahead.peek(Token![<]) && !lookahead.peek(Token![<=]) && !lookahead.peek(Token![<-]) {
            Ok(PathSegment {
                ident,
                arguments: PathArguments::AngleBracketed(input.parse()?),
            })
        } else {
            Ok(PathSegment {
                ident,
                arguments: PathArguments::None,
            })
        }
    }
}

impl ToTokens for PathSegment {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.arguments.to_tokens(tokens);
    }
}

#[derive(Clone, Debug)]
pub enum PathArguments {
    None,
    AngleBracketed(AngleBracketedGenericArguments),
}

#[derive(Clone, Debug)]
pub struct AngleBracketedGenericArguments {
    pub lt_token: Token![<],
    pub args: Punctuated<Type, Token![,]>,
    pub gt_token: Token![>],
}

impl Parse for AngleBracketedGenericArguments {
    fn parse(input: ParseStream) -> Result<Self> {
        let lt_token = input.parse()?;
        let mut args = Punctuated::new();
        loop {
            if input.peek(Token![>]) {
                break;
            }

            let arg = input.parse()?;
            args.push_value(arg);

            if input.peek(Token![>]) {
                break;
            }

            let punct = input.parse()?;
            args.push_punct(punct);
        }
        let gt_token = input.parse()?;

        Ok({
            AngleBracketedGenericArguments {
                lt_token,
                args,
                gt_token,
            }
        })
    }
}

impl ToTokens for AngleBracketedGenericArguments {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.lt_token.to_tokens(tokens);
        self.args.to_tokens(tokens);
        self.gt_token.to_tokens(tokens);
    }
}

impl ToTokens for PathArguments {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use PathArguments::*;

        match self {
            None => {}
            AngleBracketed(args) => args.to_tokens(tokens),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ParenthesizedGenericArguments {
    pub paren_token: Paren,
    pub args: Punctuated<types::Type, Token![,]>,
}

#[derive(Clone, Debug)]
pub enum GenericArgument {
    Type(Type),
    // Const(Expr),
}

impl Parse for GenericArgument {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse().map(GenericArgument::Type)
    }
}
