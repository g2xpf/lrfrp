use super::expressions::Expr;
use super::patterns::Pat;

use std::ops::Deref;

use syn::parse::{Parse, ParseStream};
use syn::token::{Eq, Let};
use syn::Result;
use syn::Token;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Stmt {
    Local(StmtLocal),
    Expr(Expr),
}

impl Stmt {
    pub fn parse_stmt(input: ParseStream) -> Result<Self> {
        use Stmt::*;
        if input.peek(Token![let]) {
            input.parse().map(Local)
        } else {
            input.parse().map(Expr)
        }
    }
}

impl Parse for Stmt {
    fn parse(input: ParseStream) -> Result<Self> {
        Stmt::parse_stmt(input)
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

impl Parse for StmtLocal {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(StmtLocal {
            let_token: input.parse()?,
            pat: input.parse()?,
            eq_token: input.parse()?,
            expr: Box::new(input.parse()?),
            semi_token: input.parse()?,
        })
    }
}
