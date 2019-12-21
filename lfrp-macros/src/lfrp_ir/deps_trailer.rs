use super::types::{Dependency, TyCtx, TyCtxRef, Var, VarEnv};
use crate::ast::expressions::{ArrowExpr, Expr, ExprPath};
use std::cell::RefCell;
use syn::Result;

pub type Context<'a, 'b, 'c, 'd> = &'a TyCtxRef<'b, 'c, 'd>;

pub struct DepExtractor<'a, 'b> {
    global: &'a VarEnv,
    lhs: Var<'b>,
}

impl<'a, 'b> DepExtractor<'a, 'b> {
    pub fn new(global: &'a VarEnv, lhs: Var<'b>) -> Self {
        DepExtractor { global, lhs }
    }

    pub fn extract<T: DepsTrailer<'b>>(
        &self,
        t: &'b mut T,
        forbid_lifted: bool,
    ) -> Result<Dependency<'b>> {
        let tcx = TyCtx::new(&self.global, &self.lhs, forbid_lifted);
        let tcx_cell = RefCell::new(tcx);
        {
            let tcx_ref = TyCtxRef::new(&tcx_cell);
            t.deps_trailer(&tcx_ref);
        }
        tcx_cell.into_inner().try_get_deps()
    }
}

pub trait DepsTrailer<'a> {
    fn deps_trailer(&'a mut self, context: Context<'_, '_, '_, 'a>);
}

impl<'a> DepsTrailer<'a> for Expr {
    fn deps_trailer(&'a mut self, context: Context<'_, '_, '_, 'a>) {
        use Expr::*;
        match self {
            Paren(e) => e.expr.deps_trailer(context),
            Binary(e) => {
                e.lhs.deps_trailer(context);
                e.rhs.deps_trailer(context);
            }
            Unary(e) => e.expr.deps_trailer(context),
            If(e) => {
                e.cond.deps_trailer(context);
                e.then_branch.deps_trailer(context);
                e.else_branch.deps_trailer(context);
            }
            Path(e) => e.deps_trailer(context),
            Lit(_) => {}

            e => unimplemented!("deps_trailer impl: {:?}", e),
        }
    }
}

impl<'a> DepsTrailer<'a> for ExprPath {
    fn deps_trailer(&'a mut self, context: Context<'_, '_, '_, 'a>) {
        context.insert_variable(&mut self.path);
    }
}

impl<'a> DepsTrailer<'a> for ArrowExpr {
    fn deps_trailer(&'a mut self, context: Context<'_, '_, '_, 'a>) {
        self.expr.deps_trailer(context)
    }
}
