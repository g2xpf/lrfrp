use syn::punctuated::Punctuated;
use syn::token::{Brace, Bracket, Colon, Comma, Dot2, Paren, Underscore};
use syn::Ident;
use syn::Member;
use syn::Token;

use super::literals::Lit;
use super::path::Path;

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

pub struct PatWild {
    pub underscore_token: Underscore,
}

pub struct PatIdent {
    pub ident: Ident,
    pub subpat: Option<(Token![@], Box<Pat>)>,
}

pub struct PatStruct {
    pub path: Path,
    pub brace_token: Brace,
    pub fields: Punctuated<FieldPat, Comma>,
    pub dot2_token: Option<Dot2>,
}

pub struct FieldPat {
    pub member: Member,
    pub colon_token: Option<Colon>,
    pub pat: Box<Pat>,
}

pub struct PatTupleStruct {
    pub path: Path,
    pub pat: PatTuple,
}

pub struct PatTuple {
    pub paren_token: Paren,
    pub front: Punctuated<Pat, Comma>,
    pub dot2_token: Option<Dot2>,
    pub comma_token: Option<Comma>,
    pub back: Punctuated<Pat, Comma>,
}

pub struct PatPath {
    pub path: Path,
}

pub struct PatLit {
    pub lit: Lit,
}

pub struct PatList {
    pub bracket_token: Bracket,
    pub front: Punctuated<Pat, Comma>,
    pub middle: Option<Box<Pat>>,
    pub dot2_token: Option<Dot2>,
    pub comma_token: Option<Comma>,
    pub back: Punctuated<Pat, Comma>,
}
