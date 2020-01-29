use super::types::{TypeLifted, Var};
use syn::Ident;

macro_rules! try_write {
    ($value:expr => $target:ident) => {{
        use syn::Error;
        if $target.is_some() {
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
pub struct UndefinedVariableError(Ident);

impl UndefinedVariableError {
    pub fn new(ident: Var) -> Self {
        UndefinedVariableError(ident.clone())
    }
}

impl Into<syn::Error> for UndefinedVariableError {
    fn into(self) -> syn::Error {
        let token = &self.0;
        let message = format!("use of undefined variable: `{}`", token.to_string());
        syn::Error::new_spanned(token, message)
    }
}

#[derive(Debug)]
pub struct MultipleDefinitionError(Ident);

impl MultipleDefinitionError {
    pub fn new(ident: Var) -> Self {
        MultipleDefinitionError(ident.clone())
    }
}

impl Into<syn::Error> for MultipleDefinitionError {
    fn into(self) -> syn::Error {
        let token = &self.0;
        let message = format!("multiple definition: `{}`", token.to_string());
        syn::Error::new_spanned(token, message)
    }
}

#[derive(Debug)]
pub struct NotCalculatedError(Ident);

impl NotCalculatedError {
    pub fn new(ident: &Ident) -> Self {
        NotCalculatedError(ident.clone())
    }
}

impl Into<syn::Error> for NotCalculatedError {
    fn into(self) -> syn::Error {
        let token = &self.0;
        let message = format!(
            "output variable `{}` won't be calculated",
            token.to_string()
        );
        syn::Error::new_spanned(token, message)
    }
}

#[derive(Debug)]
pub struct LiftedTypeNotAllowedError(Ident, TypeLifted);

impl LiftedTypeNotAllowedError {
    pub fn new(var: Var, type_lifted: &TypeLifted) -> Self {
        LiftedTypeNotAllowedError(var.clone(), type_lifted.clone())
    }
}

impl Into<syn::Error> for LiftedTypeNotAllowedError {
    fn into(self) -> syn::Error {
        let token = self.0;
        let ty = &self.1;
        let message = format!(
            "Variable `{}` has the lifted type `{}`",
            token.to_string(),
            ty.to_string()
        );
        syn::Error::new_spanned(token, message)
    }
}

#[derive(Debug)]
pub struct CyclicDependencyError(Ident);

impl CyclicDependencyError {
    pub fn new(ident: Var) -> Self {
        CyclicDependencyError(ident.clone())
    }
}

impl Into<syn::Error> for CyclicDependencyError {
    fn into(self) -> syn::Error {
        let token = &self.0;
        let message = format!(
            "cyclic dependency found in the definition `{}`",
            token.to_string(),
        );
        syn::Error::new_spanned(token, message)
    }
}
