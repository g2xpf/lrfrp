extern crate proc_macro;

mod ast;

use ast::literals::Lit;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro]
pub fn frp(input: TokenStream) -> TokenStream {
    eprintln!("{:#?}", input);
    let ast = parse_macro_input!(input as Lit);
    eprintln!("{:#?}", ast);
    let output = quote! {};
    output.into()
}
