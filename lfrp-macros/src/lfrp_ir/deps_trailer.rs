use super::types::{Dependency, TyCtxRef};
use crate::ast::expressions::{ArrowExpr, Expr};
use syn::Result;

pub type Context<'a, 'b, 'c> = &'a TyCtxRef<'b, 'c>;

pub trait DepsTrailer {
    fn deps_trailer(&mut self, varenv: Context) -> Result<Dependency>;
}

impl DepsTrailer for Expr {
    fn deps_trailer(&mut self, varenv: Context) -> Result<Dependency> {
        unimplemented!()
    }
}
impl DepsTrailer for ArrowExpr {
    fn deps_trailer(&mut self, varenv: Context) -> Result<Dependency> {
        unimplemented!()
    }
}
