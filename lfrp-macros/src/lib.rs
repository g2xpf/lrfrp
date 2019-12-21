extern crate proc_macro;

mod ast;
mod codegen;
mod lfrp_ir;
use lfrp_ir::LfrpIR;
// mod program;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro]
pub fn frp(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as ast::Ast);
    let lfrp_ir = match LfrpIR::from_ast(ast) {
        Err(e) => return e.to_compile_error().into(),
        Ok(lfrp_ir) => lfrp_ir,
    };

    codegen::codegen(lfrp_ir)
}
