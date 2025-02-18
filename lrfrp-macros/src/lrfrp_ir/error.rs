use super::types::{TypeLifted, Var};
use syn::Ident;

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
pub struct CellAsOutputError(Ident);

impl CellAsOutputError {
    pub fn new(ident: Var) -> Self {
        CellAsOutputError(ident.clone())
    }
}

impl Into<syn::Error> for CellAsOutputError {
    fn into(self) -> syn::Error {
        let token = &self.0;
        let message = format!("use delayed variable `{}` as output", token.to_string());
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
        let message = format!("output variable `{}` not calculated", token.to_string());
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
            "Variable `{}` has lifted type `{}`",
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
            "cyclic dependency found in definition `{}`",
            token.to_string(),
        );
        syn::Error::new_spanned(token, message)
    }
}
