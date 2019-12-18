extern crate proc_macro;

mod ast;
mod lfrp_ir;
use lfrp_ir::LfrpIR;
// mod program;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro]
pub fn frp(input: TokenStream) -> TokenStream {
    // eprintln!("{:#?}", input);
    let ast = parse_macro_input!(input as ast::Ast);
    eprintln!("{:#?}", ast);
    match LfrpIR::from_ast(ast) {
        Err(e) => return e.to_compile_error().into(),
        Ok(lfrp_ir) => println!("{:#?}", lfrp_ir),
    }
    // eprintln!("{:#?}", quote! { #ast });
    // let in_item = quote! { #ast };
    // in_item.into()
    (quote! {}).into()
}
