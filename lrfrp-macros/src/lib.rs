extern crate proc_macro;

mod ast;
mod codegen;
mod lrfrp_ir;

use lrfrp_ir::LrfrpIR;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro]
pub fn frp(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as ast::Ast);
    let lrfrp_ir = match LrfrpIR::from_ast(ast) {
        Err(e) => return e.to_compile_error().into(),
        Ok(lrfrp_ir) => lrfrp_ir,
    };

    codegen::codegen(lrfrp_ir)
}
