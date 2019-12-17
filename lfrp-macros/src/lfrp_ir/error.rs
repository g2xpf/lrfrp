use super::types::Var;

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
pub struct UndefinedVariableError(Var);

impl UndefinedVariableError {
    pub fn new(var: Var) -> Self {
        UndefinedVariableError(var)
    }

    pub fn generate(var: Var) -> syn::Error {
        UndefinedVariableError::new(var).into()
    }
}

impl Into<syn::Error> for UndefinedVariableError {
    fn into(self) -> syn::Error {
        let token = self.0;
        let ident = token.to_string();
        let message = format!("Undefined variable `{}`", ident);
        syn::Error::new_spanned(token, message)
    }
}
