use super::expressions::Expr;
use super::patterns::Pat;
use super::types::Type;

use std::ops::Deref;

use syn::parse::{Parse, ParseStream};
use syn::token::{Eq, Let};
use syn::Result;
use syn::Token;

#[derive(Copy, Clone)]
pub struct AllowNoSemi(pub bool);
impl Deref for AllowNoSemi {
    type Target = bool;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub enum Stmt {
    Local(StmtLocal),
    Expr(StmtExpr),
    Semi(StmtSemi),
}

impl Stmt {
    pub fn parse_stmt(input: ParseStream, allow_nosemi: AllowNoSemi) -> Result<Self> {
        unimplemented!("parse_stmt(input: ParseStream, allow_nosemi: AllowNoSemi)")
    }
}

impl Parse for Stmt {
    fn parse(input: ParseStream) -> Result<Self> {
        Stmt::parse_stmt(input, AllowNoSemi(true))
    }
}

#[derive(Debug)]
pub struct StmtLocal {
    pub let_token: Let,
    pub pat: Pat,
    pub eq_token: Eq,
    pub expr: Box<Expr>,
    pub semi_token: Token![;],
}

#[derive(Debug)]
pub struct StmtExpr {
    pub expr: Expr,
}

#[derive(Debug)]
pub struct StmtSemi {
    pub expr: Expr,
    pub semi_token: Token![;],
}
