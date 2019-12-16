use proc_macro2::Span;
use syn::{Error, Result};

macro_rules! try_write {
    ($value:expr => $target:ident) => {{
        use syn::Error;
        if let Some(_) = $target {
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
