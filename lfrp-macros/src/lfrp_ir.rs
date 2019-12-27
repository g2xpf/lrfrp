use super::ast::{self, Item};
use syn::Result;

#[macro_use]
mod error;
mod deps_check;
mod deps_trailer;
mod tsort;
mod typeck;
pub mod types;

#[derive(Debug)]
pub struct LfrpIR {
    pub module: ast::ItemMod,
    pub input: ast::ItemIn,
    pub output: ast::ItemOut,
    pub args: Option<ast::ItemArgs>,
    pub declarations: Vec<ast::Declaration>,
    pub body: deps_check::OrderedStmts,
}

impl LfrpIR {
    pub fn from_ast(ast: ast::Ast) -> Result<Self> {
        let mut module = None;
        let mut input = None;
        let mut output = None;
        let mut args = None;

        let mut declarations = vec![];
        let mut frp_stmts = vec![];

        for item in ast.items.into_iter() {
            use Item::*;
            match item {
                Mod(e) => try_write!(e => module),
                In(e) => try_write!(e => input),
                Out(e) => try_write!(e => output),
                Args(e) => try_write!(e => args),
                FrpStmt(e) => frp_stmts.push(e),
                Declaration(e) => declarations.push(e),
            }
        }

        item_unwrap!(module, "mod");
        item_unwrap!(input, "In");
        item_unwrap!(output, "Out");

        let (declarations, body) =
            deps_check::deps_check(&input, &output, &args, declarations, frp_stmts)?;

        Ok(LfrpIR {
            module,
            input,
            output,
            args,
            declarations,
            body,
        })
    }
}
