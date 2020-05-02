use super::ast::{self, Item};
use syn::Result;

mod deps_check;
mod deps_trailer;
mod error;
mod tsort;
pub mod types;

macro_rules! try_write {
    ($value:expr => $target:ident) => {{
        use syn::Error;
        if $target.is_some() {
            return Err(Error::new_spanned($value, "Duplicated items"));
        }

        $target = Some($value);
    }};
}

macro_rules! item_unwrap {
    ($value:ident, $item_name:expr) => {
        let $value = match $value {
            Some(value) => value,
            None => {
                return syn::Result::Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    format!(r#"Item `{}` not found"#, $item_name),
                ))
            }
        };
    };
}

#[derive(Debug)]
pub struct LrfrpIR {
    pub module: ast::ItemMod,
    pub input: ast::ItemIn,
    pub output: ast::ItemOut,
    pub args: Option<ast::ItemArgs>,
    pub declarations: Vec<ast::ItemDeclaration>,
    pub body: deps_check::OrderedStmts,
}

impl LrfrpIR {
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

        Ok(LrfrpIR {
            module,
            input,
            output,
            args,
            declarations,
            body,
        })
    }
}
