use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Bracket, Colon, Comma, Dot2, Paren, Underscore};
use syn::Ident;
use syn::Member;
use syn::Result;
use syn::Token;

use super::literals::Lit;
use super::path::Path;

#[derive(Debug)]
pub enum Pat {
    Wild(PatWild),
    Ident(PatIdent),
    Struct(PatStruct),
    TupleStruct(PatTupleStruct),
    Path(PatPath),
    Tuple(PatTuple),
    Lit(PatLit),
    List(PatList),
}

impl Parse for Pat {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![_]) {
            Ok(input.parse().map(Pat::Wild)?)
        // } else if lookahead.peek(Token![-]) || lookahead.peek(Lit) {
        } else if input.peek(Ident) && input.peek2(Brace) {
            unimplemented!("impl Parse for PatStruct")
        // Ok(input.parse().map(Pat::Struct)?)
        } else if input.peek(Ident) && input.peek2(Paren) {
            unimplemented!("impl Parse for PatTupleStruct")
        // Ok(input.parse().map(TupleStruct)?)
        } else if input.peek(Ident) {
            Ok(input.parse().map(Pat::Ident)?)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug)]
pub struct PatWild {
    pub underscore_token: Underscore,
}

impl Parse for PatWild {
    fn parse(input: ParseStream) -> Result<Self> {
        let underscore_token = input.parse()?;
        Ok(PatWild { underscore_token })
    }
}

#[derive(Debug)]
pub struct PatIdent {
    pub ident: Ident,
    pub subpat: Option<(Token![@], Box<Pat>)>,
}

impl Parse for PatIdent {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(PatIdent {
            ident: input.parse()?,
            subpat: {
                if input.peek(Token![@]) {
                    let at_token = input.parse()?;
                    let pat = input.parse()?;
                    Some((at_token, Box::new(pat)))
                } else {
                    None
                }
            },
        })
    }
}

#[derive(Debug)]
pub struct PatStruct {
    pub path: Path,
    pub brace_token: Brace,
    pub fields: Punctuated<FieldPat, Comma>,
    pub dot2_token: Option<Dot2>,
}

// impl Parse for PatStruct{
//     fn parse(input: ParseStream) -> Result<Self> {
//
//     }
// }

#[derive(Debug)]
pub struct FieldPat {
    pub member: Member,
    pub colon_token: Option<Colon>,
    pub pat: Box<Pat>,
}

#[derive(Debug)]
pub struct PatTupleStruct {
    pub path: Path,
    pub pat: PatTuple,
}

#[derive(Debug)]
pub struct PatTuple {
    pub paren_token: Paren,
    pub front: Punctuated<Pat, Comma>,
    pub dot2_token: Option<Dot2>,
    pub comma_token: Option<Comma>,
    pub back: Punctuated<Pat, Comma>,
}

#[derive(Debug)]
pub struct PatPath {
    pub path: Path,
}

#[derive(Debug)]
pub struct PatLit {
    pub lit: Lit,
}

#[derive(Debug)]
pub struct PatList {
    pub bracket_token: Bracket,
    pub front: Punctuated<Pat, Comma>,
    pub middle: Option<Box<Pat>>,
    pub dot2_token: Option<Dot2>,
    pub comma_token: Option<Comma>,
    pub back: Punctuated<Pat, Comma>,
}
