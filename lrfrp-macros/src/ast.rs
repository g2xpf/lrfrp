use syn::braced;
use syn::parenthesized;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Comma, Paren};
use syn::{Ident, Result, Token};

use quote::{quote, ToTokens};

use proc_macro2::TokenStream;

pub mod custom_keywords;
pub mod custom_punctuations;
pub mod expressions;
pub mod literals;
pub mod path;
pub mod patterns;
pub mod statements;
pub mod types;

#[derive(Debug)]
pub struct Ast {
    pub items: Vec<Item>,
}

#[derive(Debug)]
pub enum Item {
    Mod(ItemMod),
    In(ItemIn),
    Out(ItemOut),
    Args(ItemArgs),
    FrpStmt(ItemFrpStmt),
    Declaration(ItemDeclaration),
}

impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        use Item::*;

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![mod]) {
            Ok(input.parse().map(Mod)?)
        } else if lookahead.peek(custom_keywords::In) {
            Ok(input.parse().map(In)?)
        } else if lookahead.peek(custom_keywords::Out) {
            Ok(input.parse().map(Out)?)
        } else if lookahead.peek(custom_keywords::Args) {
            Ok(input.parse().map(Args)?)
        } else if lookahead.peek(Token![fn]) {
            Ok(input.parse().map(Declaration)?)
        } else if lookahead.peek(Token![let]) {
            Ok(input.parse().map(FrpStmt)?)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug)]
pub struct ItemFn {
    pub fn_token: Token![fn],
    pub ident: Ident,
    pub paren_token: Paren,
    pub inputs: Punctuated<FnArg, Token![,]>,
    pub right_arrow_token: Token![->],
    pub output: Box<types::Type>,
    pub eq_token: Token![=],
    pub expr: Box<expressions::Expr>,
    pub semi_token: Token![;],
}

impl Parse for ItemFn {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(ItemFn {
            fn_token: input.parse()?,
            ident: input.parse()?,
            paren_token: parenthesized!(content in input),
            inputs: content.parse_terminated(FnArg::parse)?,
            right_arrow_token: input.parse()?,
            output: input.parse()?,
            eq_token: input.parse()?,
            expr: input.parse()?,
            semi_token: input.parse()?,
        })
    }
}

impl ToTokens for ItemFn {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        let inputs = &self.inputs;
        let output = &self.output;
        let expr = &self.expr;
        tokens.extend(quote! {
            fn #ident(#inputs) -> #output {
                #expr
            }
        });
    }
}

#[derive(Debug)]
pub struct FnArg {
    pub pat: patterns::Pat,
    pub colon_token: Token![:],
    pub ty: types::Type,
}

impl Parse for FnArg {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(FnArg {
            pat: input.parse()?,
            colon_token: input.parse()?,
            ty: input.parse()?,
        })
    }
}

impl ToTokens for FnArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.pat.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.ty.to_tokens(tokens);
    }
}

#[derive(Debug)]
pub struct ItemMod {
    pub mod_token: Token![mod],
    pub name: Ident,
    pub semi_token: Token![;],
}

impl Parse for ItemMod {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ItemMod {
            mod_token: input.parse()?,
            name: input.parse()?,
            semi_token: input.parse()?,
        })
    }
}

impl ToTokens for ItemMod {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.mod_token.to_tokens(tokens);
        self.name.to_tokens(tokens);
        self.semi_token.to_tokens(tokens);
    }
}

#[derive(Debug)]
pub struct Field {
    pub ident: Ident,
    pub colon_token: Token![:],
    pub ty: types::Type,
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        let colon_token = &self.colon_token;
        let ty = &self.ty;
        tokens.extend(quote! {
            pub #ident #colon_token #ty
        });
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Field {
            ident: input.parse()?,
            colon_token: input.parse()?,
            ty: input.parse()?,
        })
    }
}

#[derive(Debug)]
pub struct ItemArgs {
    pub args_token: custom_keywords::Args,
    pub braced_token: Brace,
    pub fields: Punctuated<Field, Comma>,
}

#[derive(Debug)]
pub struct ItemOut {
    pub out_token: custom_keywords::Out,
    pub braced_token: Brace,
    pub fields: Punctuated<Field, Comma>,
}

#[derive(Debug)]
pub struct ItemIn {
    pub in_token: custom_keywords::In,
    pub braced_token: Brace,
    pub fields: Punctuated<Field, Comma>,
}

macro_rules! impl_parse_for_key {
    ($type:tt, $token_param:ident) => {
        impl Parse for $type {
            fn parse(input: ParseStream) -> Result<Self> {
                let content;
                let $token_param = input.parse()?;
                let braced_token = braced!(content in input);
                let fields = content.parse_terminated(Field::parse)?;

                Ok(Self {
                    $token_param,
                    braced_token,
                    fields,
                })
            }
        }
    }
}

impl_parse_for_key!(ItemIn, in_token);
impl_parse_for_key!(ItemOut, out_token);
impl_parse_for_key!(ItemArgs, args_token);

macro_rules! impl_to_tokens_for_key {
    ($type:tt, $token_param:ident) => {
        impl ToTokens for $type {
            fn to_tokens(&self, tokens: &mut TokenStream) {
                tokens.extend(quote! {
                    #[derive(Debug, Clone, Default)]
                    pub struct
                });
                self.$token_param.to_tokens(tokens);
                self.braced_token.surround(tokens, |tokens| {
                    self.fields.to_tokens(tokens);
                });
            }
        }
    };
}

impl_to_tokens_for_key!(ItemIn, in_token);
impl_to_tokens_for_key!(ItemOut, out_token);
impl_to_tokens_for_key!(ItemArgs, args_token);

#[derive(Debug)]
pub struct ItemStruct;
impl Parse for ItemStruct {
    fn parse(_input: ParseStream) -> Result<Self> {
        unimplemented!("impl parse for ItemStruct")
    }
}

#[derive(Debug)]
pub struct ItemEnum;
impl Parse for ItemEnum {
    fn parse(_input: ParseStream) -> Result<Self> {
        unimplemented!("impl parse for ItemEnum")
    }
}

#[derive(Debug)]
pub enum ItemDeclaration {
    Struct(ItemStruct),
    Enum(ItemEnum),
    Fn(ItemFn),
}

impl Parse for ItemDeclaration {
    fn parse(input: ParseStream) -> Result<Self> {
        use ItemDeclaration::*;

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![struct]) {
            input.parse().map(Struct)
        } else if lookahead.peek(Token![enum]) {
            input.parse().map(Enum)
        } else if lookahead.peek(Token![fn]) {
            input.parse().map(Fn)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for ItemDeclaration {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use ItemDeclaration::*;
        match self {
            Struct(_) => unimplemented!("impl ToTokens for Declaration"),
            Enum(_) => unimplemented!("impl ToTokens for Declaration"),
            Fn(e) => e.to_tokens(tokens),
        }
    }
}

#[derive(Debug)]
pub enum ItemFrpStmt {
    Dependency(FrpStmtDependency),
    Arrow(FrpStmtArrow),
}

impl Parse for ItemFrpStmt {
    fn parse(input: ParseStream) -> Result<Self> {
        use ItemFrpStmt::*;

        let let_token = input.parse()?;
        let path = input.parse()?;
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![=]) {
            let eq_token = input.parse()?;
            let expr = input.parse()?;
            let semi_token = input.parse()?;
            Ok(Dependency(FrpStmtDependency {
                let_token,
                path,
                eq_token,
                expr,
                semi_token,
            }))
        } else if lookahead.peek(Token![:]) {
            let colon_token = input.parse()?;
            let ty = input.parse()?;
            let left_arrow_token = input.parse()?;
            let arrow_expr = input.parse()?;
            let rev_arrow_token = input.parse()?;
            let expr = input.parse()?;
            let semi_token = input.parse()?;
            Ok(Arrow(FrpStmtArrow {
                let_token,
                path,
                colon_token,
                ty,
                left_arrow_token,
                arrow_expr,
                rev_arrow_token,
                expr,
                semi_token,
            }))
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug)]
pub struct FrpStmtDependency {
    pub let_token: Token![let],
    pub path: path::Path,
    pub eq_token: Token![=],
    pub expr: expressions::Expr,
    pub semi_token: Token![;],
}

impl ToTokens for FrpStmtDependency {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use crate::lrfrp_ir::types::*;
        match &self.path {
            path::Path::Segment(_) => unreachable!(),
            path::Path::TypedSegment(_, ty) => match ty {
                Type::Mono(TypeMono::Type(_)) => unreachable!(),
                Type::Lifted(TypeLifted::Signal(TypeSignal::Local(_))) => {
                    self.let_token.to_tokens(tokens)
                }
                _ => {}
            },
        }
        self.path.to_tokens(tokens);
        self.eq_token.to_tokens(tokens);
        self.expr.to_tokens(tokens);
        self.semi_token.to_tokens(tokens);
    }
}

#[derive(Debug)]
pub struct FrpStmtArrow {
    pub let_token: Token![let],
    pub path: path::Path,
    pub colon_token: Token![:],
    pub ty: types::Type,
    pub left_arrow_token: Token![<-],
    pub arrow_expr: expressions::ArrowExpr,
    pub rev_arrow_token: custom_punctuations::RevArrow,
    pub expr: expressions::Expr,
    pub semi_token: Token![;],
}

impl Parse for Ast {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut items = vec![];
        while !input.is_empty() {
            let item = input.parse()?;
            items.push(item);
        }

        Ok(Ast { items })
    }
}
