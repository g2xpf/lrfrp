use super::ast::{self, Item};
use syn::Result;

#[macro_use]
mod error;
mod codegen;
mod deps_check;
mod deps_trailer;
mod tsort;
mod typeck;
pub mod types;

#[derive(Debug)]
pub struct LfrpIR {
    module: ast::ItemMod,
    input: ast::ItemIn,
    output: ast::ItemOut,
    args: Option<ast::ItemArgs>,
    body: deps_check::OrderedStmts,
}

impl LfrpIR {
    pub fn from_ast(ast: ast::Ast) -> Result<Self> {
        let mut module = None;
        let mut input = None;
        let mut output = None;
        let mut args = None;

        let mut frp_stmts = vec![];

        for item in ast.items.into_iter() {
            use Item::*;

            match item {
                Mod(e) => try_write!(e => module),
                In(e) => try_write!(e => input),
                Out(e) => try_write!(e => output),
                Args(e) => try_write!(e => args),
                FrpStmt(e) => frp_stmts.push(e),
            }
        }

        item_unwrap!(module, "mod");
        item_unwrap!(input, "In");
        item_unwrap!(output, "Out");

        // typeck::typeck(&input, &output, &args, &mut frp_stmts)?;
        let body = deps_check::deps_check(&input, &output, &args, frp_stmts)?;

        Ok(LfrpIR {
            module,
            input,
            output,
            args,
            body,
        })
    }
}
