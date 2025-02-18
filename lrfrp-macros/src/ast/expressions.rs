use super::custom_keywords::{delay, then};
use super::literals::Lit;
use super::path::Path;
use super::patterns::Pat;
use super::statements::Stmt;
use super::types::Type;

use std::borrow::Borrow;
use std::ops::Deref;

use crate::lrfrp_ir::types;

use quote::{quote, ToTokens};

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{
    Add, And, AndAnd, Bang, Caret, Comma, Div, Dot, EqEq, Ge, Gt, Le, Lt, Ne, Or, OrOr, Rem, Shl,
    Shr, Star, Sub,
};
use syn::token::{Brace, Bracket, Colon, Dot2, FatArrow, If, Match, Paren};
use syn::Result;
use syn::Token;
use syn::{braced, bracketed, parenthesized};
use syn::{Ident, Member};

use proc_macro2::TokenStream;

mod precedence;
use precedence::Precedence;

#[derive(Debug)]
pub struct ArrowExpr {
    pub delay_token: delay,
    pub expr: Box<Expr>,
}

impl Parse for ArrowExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ArrowExpr {
            delay_token: input.parse()?,
            expr: Box::new(input.parse()?),
        })
    }
}

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
    #[allow(dead_code)]
    Match(ExprMatch),
    Paren(ExprParen),
    Struct(ExprStruct),
    Tuple(ExprTuple),
    Path(ExprPath),
    List(ExprList),
    Type(ExprType),

    // types for type checker
    #[allow(dead_code)]
    TypedExpr(Box<Expr>, types::Type),
}

impl Parse for Expr {
    fn parse(input: ParseStream) -> Result<Self> {
        ambiguous_expr(input, AllowStruct(true))
    }
}

impl ToTokens for Expr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Expr::*;
        match self {
            Unary(e) => e.to_tokens(tokens),
            Binary(e) => e.to_tokens(tokens),
            // Block(ref e),
            Call(e) => e.to_tokens(tokens),
            // Field(ref e),
            // Cast(ref e),
            If(e) => e.to_tokens(tokens),
            // Index(ref e),
            Lit(e) => e.to_tokens(tokens),
            // Match(ref e),
            Paren(e) => e.to_tokens(tokens),
            // Struct(ref e),
            // Tuple(ref e),
            Path(e) => e.to_tokens(tokens),
            // List(ref e),
            // Type(ref e),
            TypedExpr(e, _) => e.to_tokens(tokens),

            Block(e) => e.to_tokens(tokens),

            _ => unimplemented!("totokens for expr"),
        }
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
    // needs to exclude keyword
    } else if input.peek(delay) || input.peek(then) {
        Err(input.error("keywords are not allowed"))
    } else if input.peek(Ident) {
        path_or_struct_or_fn(input, allow_struct)
    } else if input.peek(Paren) {
        paren_or_tuple(input)
    } else if input.peek(Bracket) {
        input.parse().map(Expr::List)
    } else if input.peek(Token![if]) {
        input.parse().map(Expr::If)
    } else if input.peek(Match) {
        unimplemented!("Match")
    } else if input.peek(Brace) {
        input.parse().map(Expr::Block)
    } else {
        Err(input.error("unexpected expression"))
    }
}

fn path_or_struct_or_fn(input: ParseStream, allow_struct: AllowStruct) -> Result<Expr> {
    let path: ExprPath = input.parse()?;
    if *allow_struct && input.peek(Brace) {
        let content;
        let brace_token = braced!(content in input);
        let mut fields = Punctuated::new();
        loop {
            if content.fork().parse::<Member>().is_err() {
                break;
            }
            let field = content.parse()?;
            fields.push_value(field);
            if !content.peek(Token![,]) {
                break;
            }
            let comma_token = content.parse()?;
            fields.push_punct(comma_token);
        }

        let (dot2_token, rest) = if fields.empty_or_trailing() && content.peek(Token![..]) {
            let dot2_token: Token![..] = content.parse()?;
            let rest: Expr = content.parse()?;
            (Some(dot2_token), Some(Box::new(rest)))
        } else {
            (None, None)
        };

        Ok(Expr::Struct(ExprStruct {
            path: path.path,
            brace_token,
            fields,
            dot2_token,
            rest,
        }))
    } else if input.peek(Paren) {
        let content;
        Ok(Expr::Call(ExprCall {
            func: path,
            paren_token: parenthesized!(content in input),
            args: content.parse_terminated(Expr::parse)?,
        }))
    } else {
        Ok(Expr::Path(path))
    }
}

fn paren_or_tuple(input: ParseStream) -> Result<Expr> {
    let content;
    let paren_token = parenthesized!(content in input);
    if content.is_empty() {
        return Ok(Expr::Tuple(ExprTuple {
            paren_token,
            elems: Punctuated::new(),
        }));
    }

    let first: Expr = content.parse()?;
    if content.is_empty() {
        return Ok(Expr::Paren(ExprParen {
            paren_token,
            expr: Box::new(first),
        }));
    }

    let mut elems = Punctuated::new();
    elems.push_value(first);
    while !content.is_empty() {
        let punct = content.parse()?;
        elems.push_punct(punct);
        if content.is_empty() {
            break;
        }
        let value = content.parse()?;
        elems.push_value(value);
    }
    Ok(Expr::Tuple(ExprTuple { paren_token, elems }))
}

fn trailer_helper(input: ParseStream, mut e: Expr) -> Result<Expr> {
    loop {
        if input.peek(Token![.]) && input.peek(Token![..]) {
            let dot_token = input.parse()?;
            let member = input.parse()?;
            e = Expr::Field(ExprField {
                base: Box::new(e),
                dot_token,
                member,
            });
        } else if input.peek(Bracket) {
            let content;
            e = Expr::Index(ExprIndex {
                expr: Box::new(e),
                bracket_token: bracketed!(content in input),
                index: content.parse()?,
            });
        } else {
            break;
        }
    }
    Ok(e)
}

fn parse_expr(
    input: ParseStream,
    mut lhs: Expr,
    allow_struct: AllowStruct,
    base: Precedence,
) -> Result<Expr> {
    loop {
        if input
            .fork()
            .parse::<BinOp>()
            .ok()
            .map_or(false, |op| Precedence::of(&op) >= base)
        {
            let op: BinOp = input.parse()?;
            let precedence = Precedence::of(&op);
            let mut rhs = unary_expr(input, allow_struct)?;
            loop {
                let next = Precedence::peek(input);
                if precedence < next {
                    rhs = parse_expr(input, rhs, allow_struct, next)?;
                } else {
                    break;
                }
            }
            lhs = Expr::Binary(ExprBinary {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            });
        } else if Precedence::Cast >= base && input.peek(Token![as]) {
            let as_token: Token![as] = input.parse()?;
            let ty = input.parse()?;
            lhs = Expr::Cast(ExprCast {
                expr: Box::new(lhs),
                as_token,
                ty: Box::new(ty),
            });
        } else if Precedence::Cast >= base && input.peek(Token![:]) && !input.peek(Token![::]) {
            let colon_token: Token![:] = input.parse()?;
            let ty = input.parse()?;
            lhs = Expr::Type(ExprType {
                expr: Box::new(lhs),
                colon_token,
                ty: Box::new(ty),
            });
        } else {
            break;
        }
    }
    Ok(lhs)
}

#[derive(Debug)]
pub struct ExprPath {
    pub path: Path,
}

impl Borrow<Ident> for ExprPath {
    fn borrow(&self) -> &Ident {
        self.path.borrow()
    }
}

impl Parse for ExprPath {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ExprPath {
            path: input.parse()?,
        })
    }
}

impl ToTokens for ExprPath {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.path.to_tokens(tokens);
    }
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

impl ToTokens for ExprParen {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.paren_token
            .surround(tokens, |tokens| self.expr.to_tokens(tokens))
    }
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

impl Parse for FieldValue {
    fn parse(input: ParseStream) -> Result<Self> {
        let member: Member = input.parse()?;
        let (colon_token, value) = match member {
            Member::Unnamed(_) => {
                let colon_token: Token![:] = input.parse()?;
                let value: Expr = input.parse()?;
                (Some(colon_token), value)
            }
            Member::Named(ref ident) => {
                let value = Expr::Path(ExprPath {
                    path: Path::from(ident.clone()),
                });
                (None, value)
            }
        };

        Ok(FieldValue {
            member,
            colon_token,
            expr: value,
        })
    }
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
    pub lit: Lit,
}

impl Parse for ExprLit {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ExprLit {
            lit: input.parse()?,
        })
    }
}

impl ToTokens for ExprLit {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.lit.to_tokens(tokens)
    }
}

#[derive(Debug)]
pub struct ExprIndex {
    pub expr: Box<Expr>,
    pub bracket_token: Bracket,
    pub index: Box<Expr>,
}

#[derive(Debug)]
pub struct ExprIf {
    pub if_token: Token![if],
    pub cond: Box<Expr>,
    pub then_token: then,
    pub then_branch: Box<Expr>,
    pub else_token: Token![else],
    pub else_branch: Box<Expr>,
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

impl ToTokens for ExprIf {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let cond = &self.cond;
        let then_branch = &self.then_branch;
        let else_branch = &self.else_branch;
        tokens.extend(quote! {
            if #cond {
                #then_branch
            } else {
                #else_branch
            }
        });
    }
}

#[derive(Debug)]
pub struct ExprCast {
    expr: Box<Expr>,
    as_token: Token![as],
    ty: Box<Type>,
}

#[derive(Debug)]
pub struct ExprType {
    expr: Box<Expr>,
    colon_token: Token![:],
    ty: Box<Type>,
}

#[derive(Debug)]
pub struct ExprBlock {
    pub braced_token: Brace,
    pub stmts: Vec<Stmt>,
}

impl Parse for ExprBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(ExprBlock {
            braced_token: braced!(content in input),
            stmts: {
                let mut stmts = Vec::new();

                loop {
                    while content.peek(Token![;]) {
                        content.parse::<Token![;]>()?;
                    }
                    if content.is_empty() {
                        break;
                    }
                    // No expressions require semicolon in order to be statements
                    let stmt = Stmt::parse_stmt(&content)?;
                    let is_final_stmt = if let Stmt::Expr(_) = &stmt {
                        true
                    } else {
                        false
                    };
                    stmts.push(stmt);
                    if content.is_empty() {
                        break;
                    } else if is_final_stmt {
                        return Err(content.error("unexpected token"));
                    }
                }
                stmts
            },
        })
    }
}

impl ToTokens for ExprBlock {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.braced_token.surround(tokens, |tokens| {
            self.stmts.iter().for_each(|stmt| stmt.to_tokens(tokens));
        });
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
    pub op: UnOp,
    pub expr: Box<Expr>,
}

impl Parse for ExprUnary {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ExprUnary {
            op: input.parse()?,
            expr: Box::new(input.parse()?),
        })
    }
}

impl ToTokens for ExprUnary {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.op.to_tokens(tokens);
        self.expr.to_tokens(tokens);
    }
}

#[derive(Debug)]
pub struct ExprBinary {
    pub lhs: Box<Expr>,
    pub op: BinOp,
    pub rhs: Box<Expr>,
}

impl ToTokens for ExprBinary {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.lhs.to_tokens(tokens);
        self.op.to_tokens(tokens);
        self.rhs.to_tokens(tokens);
    }
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

impl ToTokens for UnOp {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use UnOp::*;
        match self {
            Not(ref op) => op.to_tokens(tokens),
            Neg(ref op) => op.to_tokens(tokens),
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
            elems: Punctuated::<Expr, Token![,]>::parse_terminated(&content)?,
        })
    }
}

#[derive(Debug)]
pub struct ExprCall {
    pub func: ExprPath,
    pub paren_token: Paren,
    pub args: Punctuated<Expr, Comma>,
}

impl ToTokens for ExprCall {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.func.to_tokens(tokens);
        self.paren_token.surround(tokens, |tokens| {
            self.args.to_tokens(tokens);
        });
    }
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
}

impl Parse for BinOp {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![&&]) {
            input.parse().map(BinOp::And)
        } else if input.peek(Token![||]) {
            input.parse().map(BinOp::Or)
        } else if input.peek(Token![<<]) {
            input.parse().map(BinOp::Shl)
        } else if input.peek(Token![>>]) {
            input.parse().map(BinOp::Shr)
        } else if input.peek(Token![==]) {
            input.parse().map(BinOp::Eq)
        } else if input.peek(Token![<=]) {
            input.parse().map(BinOp::Le)
        } else if input.peek(Token![!=]) {
            input.parse().map(BinOp::Ne)
        } else if input.peek(Token![>=]) {
            input.parse().map(BinOp::Ge)
        } else if input.peek(Token![+]) {
            input.parse().map(BinOp::Add)
        // prevent from matching `-<` token
        } else if input.peek(Token![-]) && !input.peek2(Token![<]) {
            input.parse().map(BinOp::Sub)
        } else if input.peek(Token![*]) {
            input.parse().map(BinOp::Mul)
        } else if input.peek(Token![/]) {
            input.parse().map(BinOp::Div)
        } else if input.peek(Token![%]) {
            input.parse().map(BinOp::Rem)
        } else if input.peek(Token![^]) {
            input.parse().map(BinOp::BitXor)
        } else if input.peek(Token![&]) {
            input.parse().map(BinOp::BitAnd)
        } else if input.peek(Token![|]) {
            input.parse().map(BinOp::BitOr)
        } else if input.peek(Token![<]) {
            input.parse().map(BinOp::Lt)
        } else if input.peek(Token![>]) {
            input.parse().map(BinOp::Gt)
        } else {
            Err(input.error("expected binary operator"))
        }
    }
}

impl ToTokens for BinOp {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use BinOp::*;
        match self {
            Add(e) => e.to_tokens(tokens),
            Sub(e) => e.to_tokens(tokens),
            Mul(e) => e.to_tokens(tokens),
            Div(e) => e.to_tokens(tokens),
            Rem(e) => e.to_tokens(tokens),
            And(e) => e.to_tokens(tokens),
            Or(e) => e.to_tokens(tokens),
            BitXor(e) => e.to_tokens(tokens),
            BitAnd(e) => e.to_tokens(tokens),
            BitOr(e) => e.to_tokens(tokens),
            Shl(e) => e.to_tokens(tokens),
            Shr(e) => e.to_tokens(tokens),
            Eq(e) => e.to_tokens(tokens),
            Lt(e) => e.to_tokens(tokens),
            Le(e) => e.to_tokens(tokens),
            Ne(e) => e.to_tokens(tokens),
            Ge(e) => e.to_tokens(tokens),
            Gt(e) => e.to_tokens(tokens),
        }
    }
}
