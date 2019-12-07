use syn::parse::{Parse, ParseStream};
use syn::Lit as L;
use syn::Result;
use syn::{LitBool, LitFloat, LitInt};

#[derive(Debug)]
pub enum Lit {
    Int(LitInt),
    Float(LitFloat),
    Bool(LitBool),
}

impl Parse for Lit {
    fn parse(input: ParseStream) -> Result<Self> {
        // literal suffixes will be filtered
        // after parsing
        input.step(|cursor| {
            if let Some((lit, rest)) = cursor.literal() {
                let lit = L::new(lit);
                if let L::Int(int) = lit {
                    return Ok((Lit::Int(int), rest));
                }
                if let L::Float(float) = lit {
                    return Ok((Lit::Float(float), rest));
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
