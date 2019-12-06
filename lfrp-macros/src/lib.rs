extern crate proc_macro;

mod ast;

use ast::Ast;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro]
pub fn frp(input: TokenStream) -> TokenStream {
    println!("{:#?}", input);
    let ast = parse_macro_input!(input as Ast);
    println!("{:#?}", ast);
    let output = quote! {};
    output.into()
}
