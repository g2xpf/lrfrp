use syn::parse::{Parse, ParseStream};
use syn::Lit as L;
use syn::Result;
use syn::{LitBool, LitFloat, LitInt};

use quote::ToTokens;

use proc_macro2::TokenStream;

#[derive(Debug)]
pub enum Lit {
    Int(LitInt),
    Float(LitFloat),
    Bool(LitBool),
}

impl Lit {
    pub fn peeked(input: &ParseStream) -> bool {
        input.peek(LitInt) || input.peek(LitFloat) || {
            let cursor = input.cursor();
            if let Some((ident, _)) = cursor.ident() {
                ident == "True" || ident == "False"
            } else {
                false
            }
        }
    }
}

impl Parse for Lit {
    fn parse(input: ParseStream) -> Result<Self> {
        input.step(|cursor| {
            if let Some((lit, rest)) = cursor.literal() {
                let lit = L::new(lit);
                if let L::Int(int) = lit {
                    match int.suffix() {
                        "" | "Int" => return Ok((Lit::Int(int), rest)),
                        _ => return Err(cursor.error("unexpected suffix")),
                    }
                }
                if let L::Float(float) = lit {
                    match float.suffix() {
                        "" | "Float" => return Ok((Lit::Float(float), rest)),
                        _ => return Err(cursor.error("unexpected suffix")),
                    }
                }
            }

            #[allow(clippy::never_loop)]
            while let Some((ident, rest)) = cursor.ident() {
                // use `if` instead of `match`
                // because proc_macro2::Ident: PartialEq<T: AsRef<str>>
                let value = if ident == "True" {
                    true
                } else if ident == "False" {
                    false
                } else {
                    break;
                };
                let lit_bool = LitBool {
                    value,
                    span: ident.span(),
                };
                return Ok((Lit::Bool(lit_bool), rest));
            }

            Err(cursor.error("unexpected literal"))
        })
    }
}

impl ToTokens for Lit {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Lit::*;
        match self {
            Float(e) => e.to_tokens(tokens),
            Int(e) => e.to_tokens(tokens),
            Bool(e) => e.to_tokens(tokens),
        }
    }
}
