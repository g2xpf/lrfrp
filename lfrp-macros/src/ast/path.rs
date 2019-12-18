use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::Ident;
use syn::Result;
use syn::Token;

use quote::ToTokens;

use proc_macro2::TokenStream;

use super::types;

#[derive(Clone, Debug)]
pub struct Path {
    pub segment: PathSegment,
}

impl Path {
    pub fn get_ident(&self) -> &Ident {
        self.segment.get_ident()
    }
}

impl<T> From<T> for Path
where
    T: Into<PathSegment>,
{
    fn from(t: T) -> Self {
        Path { segment: t.into() }
    }
}

impl Parse for Path {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Path {
            segment: input.parse()?,
        })
    }
}

impl ToTokens for Path {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.segment.to_tokens(tokens);
    }
}

#[derive(Clone, Debug)]
pub struct PathSegment {
    pub ident: Ident,
    pub arguments: PathArguments,
}

impl PathSegment {
    pub fn get_ident(&self) -> &Ident {
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
    pub args: Punctuated<types::Type, Token![,]>,
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
    Type(types::Type),
    // Const(Expr),
}

impl Parse for GenericArgument {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse().map(GenericArgument::Type)
    }
}
