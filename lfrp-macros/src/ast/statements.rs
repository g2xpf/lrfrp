use super::expressions::Expr;
use super::patterns::Pat;

use syn::token::Eq;
use syn::Token;

pub enum Stmt {
    Local(StmtLocal),
    Expr(StmtExpr),
    Semi(StmtSemi),
}

pub struct StmtLocal {
    pub pat: Box<Pat>,
    pub eq_token: Eq,
    pub expr: Box<Expr>,
    pub semi_token: Token![;],
}

pub struct StmtExpr {
    pub expr: Expr,
}

pub struct StmtSemi {
    pub expr: Expr,
    pub semi_token: Token![;],
}
