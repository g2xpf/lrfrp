use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Result, Token};

#[derive(Debug)]
pub struct Ast {
    pub item_mod: ItemMod,
}

impl Parse for Ast {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut item_mod = None;

        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![mod]) {
                item_mod = Some(input.parse()?);
            } else {
            }
        }

        if let (Some(item_mod)) = (item_mod) {
            Ok(Ast { item_mod })
        } else {
            unimplemented!()
        }
    }
}

#[derive(Debug)]
pub struct ItemMod {
    pub mod_token: Token![mod],
    pub name: Ident,
    pub semi_token: Token![;],
}

impl Parse for ItemMod {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ItemMod {
            mod_token: input.parse()?,
            name: input.parse()?,
            semi_token: input.parse()?,
        })
    }
}
