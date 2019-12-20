use super::types::{TypeLifted, Var};
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

#[derive(Debug)]
pub struct NotCalculatedError<'a>(&'a Ident);

impl<'a> NotCalculatedError<'a> {
    pub fn new(ident: &'a Ident) -> Self {
        NotCalculatedError(ident)
    }
}

impl<'a> Into<syn::Error> for NotCalculatedError<'a> {
    fn into(self) -> syn::Error {
        let token = &self.0;
        let message = format!(
            "Output variable `{}` won't be calculated",
            token.to_string()
        );
        syn::Error::new_spanned(token, message)
    }
}

#[derive(Debug)]
pub struct LiftedTypeNotAllowedError<'a>(Var<'a>, &'a TypeLifted);

impl<'a> LiftedTypeNotAllowedError<'a> {
    pub fn new(var: Var<'a>, type_lifted: &'a TypeLifted) -> Self {
        LiftedTypeNotAllowedError(var, type_lifted)
    }
}

impl<'a> Into<syn::Error> for LiftedTypeNotAllowedError<'a> {
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
pub struct CyclicDependencyError<'a>(pub Var<'a>);

impl<'a> CyclicDependencyError<'a> {
    pub fn new(ident: &'a Var) -> Self {
        CyclicDependencyError(ident)
    }
}

impl<'a> Into<syn::Error> for CyclicDependencyError<'a> {
    fn into(self) -> syn::Error {
        let token = &self.0;
        let message = format!(
            "cyclic dependency found in the definition `{}`",
            token.to_string(),
        );
        syn::Error::new_spanned(token, message)
    }
}
