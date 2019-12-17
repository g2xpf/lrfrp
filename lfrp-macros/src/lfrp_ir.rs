use super::ast::{self, Item, ItemFrpStmt};
use syn::Result;

#[macro_use]
mod error;
mod deps_check;
mod types;

use deps_check::deps_check;

#[derive(Debug)]
pub struct LfrpIR {
    module: ast::ItemMod,
    input: ast::ItemIn,
    output: ast::ItemOut,
    args: Option<ast::ItemArgs>,
    body: Vec<ast::ItemFrpStmt>,
}

impl LfrpIR {
    pub fn from_ast(ast: ast::Ast) -> Result<Self> {
        let mut module = None;
        let mut input = None;
        let mut output = None;
        let mut args = None;

        let mut deps = vec![];
        let mut arrows = vec![];

        for item in ast.items.into_iter() {
            use Item::*;
            use ItemFrpStmt::*;

            match item {
                Mod(e) => try_write!(e => module),
                In(e) => try_write!(e => input),
                Out(e) => try_write!(e => output),
                Args(e) => try_write!(e => args),
                FrpStmt(e) => match e {
                    Dependency(e) => deps.push(e),
                    Arrow(e) => arrows.push(e),
                },
            }
        }

        item_unwrap!(module, "mod");
        item_unwrap!(input, "In");
        item_unwrap!(output, "Out");

        let body = deps_check(&module, &input, &output, &args, deps, arrows)?;

        Ok(LfrpIR {
            module,
            input,
            output,
            args,
            body,
        })
    }
}
