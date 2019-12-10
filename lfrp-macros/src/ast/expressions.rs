use super::custom_keywords::then;
use super::custom_punctuations::StarStar;
use super::literals::Lit;
use super::path::Path;
use super::patterns::Pat;
use super::statements::Stmt;
use super::types::Type;

use syn::punctuated::Punctuated;
use syn::token::{
    Add, And, AndAnd, Bang, Caret, Comma, Div, Dot, EqEq, Ge, Gt, Le, Lt, Ne, Or, OrOr, Rem, Shl,
    Shr, Star, Sub,
};
use syn::token::{Brace, Bracket, Colon, Dot2, Else, FatArrow, If, Match, Paren};
use syn::{Member, Token};

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
}
pub struct ExprPath {
    path: Path,
}

pub struct ExprTuple {
    paren_token: Paren,
    elems: Punctuated<Expr, Token![,]>,
}

pub struct ExprParen {
    pub paren_token: Paren,
    pub expr: Box<Expr>,
}

pub struct ExprStruct {
    pub path: Path,
    pub brace_token: Brace,
    pub fields: Punctuated<FieldValue, Comma>,
    pub dot2_token: Option<Dot2>,
    pub rest: Option<Box<Expr>>,
}

pub struct FieldValue {
    pub member: Member,
    pub colon_token: Option<Colon>,
    pub expr: Expr,
}

pub struct ExprMatch {
    pub match_token: Match,
    pub expr: Box<Expr>,
    pub brace_token: Brace,
    pub arms: Vec<Arm>,
}

pub struct Arm {
    pub pat: Pat,
    pub guard: Option<(If, Box<Expr>)>,
    pub fat_arrow_token: FatArrow,
    pub body: Box<Expr>,
    pub comma: Option<Comma>,
}

pub struct ExprLit {
    lit: Lit,
}

pub struct ExprIndex {
    expr: Box<Expr>,
    braced_token: Bracket,
    index: Box<Expr>,
}

pub struct ExprIf {
    if_token: If,
    cond: Box<Expr>,
    then_token: then,
    then_branch: Box<Expr>,
    else_token: Else,
    else_branch: Box<Expr>,
}

pub struct ExprCast {
    expr: Box<Expr>,
    as_token: Token![as],
    ty: Type,
}

pub struct ExprBlock {
    braced_token: Brace,
    stmts: Vec<Stmt>,
}

pub struct ExprField {
    pub base: Box<Expr>,
    pub dot_token: Dot,
    pub member: Member,
}

pub struct ExprUnary {
    op: UnOp,
    expr: Box<Expr>,
}

pub struct ExprBinary {
    lhs: Box<Expr>,
    op: BinaryOp,
    rhs: Box<Expr>,
}

pub enum UnOp {
    Not(Bang),
    Neg(Sub),
}
pub struct ExprCall {
    pub func: Box<Expr>,
    pub paren_token: Paren,
    pub args: Punctuated<Expr, Comma>,
}

pub enum BinaryOp {
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
