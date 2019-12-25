use super::expressions::Expr;
use super::patterns::Pat;

use syn::parse::{Parse, ParseStream};
use syn::token::{Eq, Let};
use syn::Result;
use syn::Token;

use proc_macro2::TokenStream;
use quote::ToTokens;

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

impl ToTokens for Stmt {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Stmt::Local(s) => s.to_tokens(tokens),
            Stmt::Expr(e) => e.to_tokens(tokens),
        }
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

impl ToTokens for StmtLocal {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.let_token.to_tokens(tokens);
        self.pat.to_tokens(tokens);
        self.eq_token.to_tokens(tokens);
        self.expr.to_tokens(tokens);
        self.semi_token.to_tokens(tokens);
    }
}
