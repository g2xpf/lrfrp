use super::custom_keywords::then;
use super::custom_punctuations::StarStar;
use super::literals::Lit;
use super::path::Path;
use super::patterns::Pat;
use super::statements::Stmt;
use super::types::Type;
use std::ops::Deref;
use syn::parse::{Parse, ParseStream};
use syn::Ident;
use syn::Result;
use syn::{braced, bracketed};

use syn::punctuated::Punctuated;
use syn::token::{
    Add, And, AndAnd, Bang, Caret, Comma, Div, Dot, EqEq, Ge, Gt, Le, Lt, Ne, Or, OrOr, Rem, Shl,
    Shr, Star, Sub,
};
use syn::token::{Brace, Bracket, Colon, Dot2, Else, FatArrow, If, Match, Paren};
use syn::{Member, Token};

mod precedence;
use precedence::Precedence;

#[derive(Copy, Clone)]
struct AllowStruct(bool);
impl Deref for AllowStruct {
    type Target = bool;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub enum Expr {
    Unary(ExprUnary),
    Binary(ExprBinary),
    Block(ExprBlock),
    Call(ExprCall),
    Field(ExprField),
    Cast(ExprCast),
    If(ExprIf),
    Index(ExprIndex),
    Lit(ExprLit),
    Match(ExprMatch),
    Paren(ExprParen),
    Struct(ExprStruct),
    Tuple(ExprTuple),
    Path(ExprPath),
    List(ExprList),
}

impl Parse for Expr {
    fn parse(input: ParseStream) -> Result<Self> {
        ambiguous_expr(input, AllowStruct(true))
    }
}

fn ambiguous_expr(input: ParseStream, allow_struct: AllowStruct) -> Result<Expr> {
    let lhs = unary_expr(input, allow_struct)?;
    parse_expr(input, lhs, allow_struct, Precedence::Any)
}

fn unary_expr(input: ParseStream, allow_struct: AllowStruct) -> Result<Expr> {
    if input.peek(Token![!]) || input.peek(Token![-]) {
        Ok(input.parse().map(Expr::Unary)?)
    } else {
        trailer_expr(input, allow_struct)
    }
}

fn trailer_expr(input: ParseStream, allow_struct: AllowStruct) -> Result<Expr> {
    let atom = atom_expr(input, allow_struct)?;
    trailer_helper(input, atom)
}

fn atom_expr(input: ParseStream, allow_struct: AllowStruct) -> Result<Expr> {
    if Lit::peeked(&input) {
        // substitution for `input.peek(literals::Lit)`

        input.parse().map(Expr::Lit)
    } else if input.peek(Ident) {
        path_or_struct(input)
    } else if input.peek(Paren) {
        paren_or_tuple(input)
    } else if input.peek(Bracket) {
        input.parse().map(Expr::List)
    } else if input.peek(If) {
        input.parse().map(Expr::If)
    } else if input.peek(Match) {
        unimplemented!()
    } else if input.peek(Brace) {
        input.parse().map(Expr::Block)
    } else {
        Err(input.error("unexpected expression"))
    }
}

fn path_or_struct(input: ParseStream) -> Result<Expr> {
    unimplemented!()
}

fn paren_or_tuple(input: ParseStream) -> Result<Expr> {
    unimplemented!()
}

fn trailer_helper(input: ParseStream, mut e: Expr) -> Result<Expr> {
    unimplemented!()
}

fn parse_expr(
    input: ParseStream,
    lhs: Expr,
    allow_struct: AllowStruct,
    base: Precedence,
) -> Result<Expr> {
    unimplemented!()
}

#[derive(Debug)]
pub struct ExprPath {
    path: Path,
}

#[derive(Debug)]
pub struct ExprTuple {
    paren_token: Paren,
    elems: Punctuated<Expr, Token![,]>,
}

#[derive(Debug)]
pub struct ExprParen {
    pub paren_token: Paren,
    pub expr: Box<Expr>,
}

#[derive(Debug)]
pub struct ExprStruct {
    pub path: Path,
    pub brace_token: Brace,
    pub fields: Punctuated<FieldValue, Comma>,
    pub dot2_token: Option<Dot2>,
    pub rest: Option<Box<Expr>>,
}

#[derive(Debug)]
pub struct FieldValue {
    pub member: Member,
    pub colon_token: Option<Colon>,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct ExprMatch {
    pub match_token: Match,
    pub expr: Box<Expr>,
    pub brace_token: Brace,
    pub arms: Vec<Arm>,
}

#[derive(Debug)]
pub struct Arm {
    pub pat: Pat,
    pub guard: Option<(If, Box<Expr>)>,
    pub fat_arrow_token: FatArrow,
    pub body: Box<Expr>,
    pub comma: Option<Comma>,
}

#[derive(Debug)]
pub struct ExprLit {
    lit: Lit,
}

impl Parse for ExprLit {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ExprLit {
            lit: input.parse()?,
        })
    }
}

#[derive(Debug)]
pub struct ExprIndex {
    expr: Box<Expr>,
    braced_token: Bracket,
    index: Box<Expr>,
}

#[derive(Debug)]
pub struct ExprIf {
    if_token: If,
    cond: Box<Expr>,
    then_token: then,
    then_branch: Box<Expr>,
    else_token: Else,
    else_branch: Box<Expr>,
}

impl Parse for ExprIf {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ExprIf {
            if_token: input.parse()?,
            cond: Box::new(input.parse()?),
            then_token: input.parse()?,
            then_branch: Box::new(input.parse()?),
            else_token: input.parse()?,
            else_branch: Box::new(input.parse()?),
        })
    }
}

#[derive(Debug)]
pub struct ExprCast {
    expr: Box<Expr>,
    as_token: Token![as],
    ty: Type,
}

#[derive(Debug)]
pub struct ExprBlock {
    braced_token: Brace,
    stmts: Vec<Stmt>,
}

impl Parse for ExprBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(ExprBlock {
            braced_token: braced!(content in input),
            stmts: {
                let mut stmts = Vec::new();

                loop {
                    while input.peek(Token![;]) {
                        input.parse::<Token![;]>()?;
                    }
                    if input.is_empty() {
                        break;
                    }
                    // No expressions require semicolon in order to be a statement
                    let stmt = Stmt::parse_stmt(input, true)?;
                    stmts.push(stmt);
                    if input.is_empty() {
                        break;
                    }
                }
                stmts
            },
        })
    }
}

#[derive(Debug)]
pub struct ExprField {
    pub base: Box<Expr>,
    pub dot_token: Dot,
    pub member: Member,
}

#[derive(Debug)]
pub struct ExprUnary {
    op: UnOp,
    expr: Box<Expr>,
}

impl Parse for ExprUnary {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ExprUnary {
            op: input.parse()?,
            expr: Box::new(input.parse()?),
        })
    }
}

#[derive(Debug)]
pub struct ExprBinary {
    lhs: Box<Expr>,
    op: BinOp,
    rhs: Box<Expr>,
}

#[derive(Debug)]
pub enum UnOp {
    Not(Bang),
    Neg(Sub),
}

impl Parse for UnOp {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![!]) {
            Ok(input.parse().map(UnOp::Not)?)
        } else if lookahead.peek(Token![-]) {
            Ok(input.parse().map(UnOp::Neg)?)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug)]
pub struct ExprList {
    pub bracket_token: Bracket,
    pub elems: Punctuated<Expr, Token![,]>,
}

impl Parse for ExprList {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(ExprList {
            bracket_token: bracketed!(content in input),
            elems: Punctuated::<Expr, Token![,]>::parse_terminated(input)?,
        })
    }
}

#[derive(Debug)]
pub struct ExprCall {
    pub func: Box<Expr>,
    pub paren_token: Paren,
    pub args: Punctuated<Expr, Comma>,
}

#[derive(Debug)]
pub enum BinOp {
    Add(Add),
    Sub(Sub),
    Mul(Star),
    Div(Div),
    Rem(Rem),
    And(AndAnd),
    Or(OrOr),
    BitXor(Caret),
    BitAnd(And),
    BitOr(Or),
    Shl(Shl),
    Shr(Shr),
    Eq(EqEq),
    Lt(Lt),
    Le(Le),
    Ne(Ne),
    Ge(Ge),
    Gt(Gt),
    Pow(StarStar),
}
