use super::types::Var;
use syn::Ident;

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

#[derive(Debug)]
pub struct UndefinedVariableError<'a>(&'a Ident);

impl<'a> UndefinedVariableError<'a> {
    pub fn new(ident: &'a Ident) -> Self {
        UndefinedVariableError(ident)
    }
}

impl<'a> Into<syn::Error> for UndefinedVariableError<'a> {
    fn into(self) -> syn::Error {
        let token = &self.0;
        let message = format!("Undefined variable `{}`", token.to_string());
        syn::Error::new_spanned(token, message)
    }
}

#[derive(Debug)]
pub struct MultipleDefinitionError<'a>(&'a Ident);

impl<'a> MultipleDefinitionError<'a> {
    pub fn new(ident: &'a Ident) -> Self {
        MultipleDefinitionError(ident)
    }
}

impl<'a> Into<syn::Error> for MultipleDefinitionError<'a> {
    fn into(self) -> syn::Error {
        let token = &self.0;
        let message = format!("Multiple definition `{}`", token.to_string());
        syn::Error::new_spanned(token, message)
    }
}
