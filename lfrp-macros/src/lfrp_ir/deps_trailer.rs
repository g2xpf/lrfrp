use super::types::{Dependency, TyCtx, TyCtxRef, Var, VarEnv};
use crate::ast::expressions::{
    ArrowExpr, Expr, ExprBinary, ExprBlock, ExprCall, ExprCast, ExprField, ExprIf, ExprIndex,
    ExprList, ExprLit, ExprMatch, ExprParen, ExprPath, ExprStruct, ExprTuple, ExprType, ExprUnary,
};
use std::cell::RefCell;
use syn::Result;

pub type Context<'a, 'b, 'c> = &'a TyCtxRef<'b, 'c>;

pub struct DepExtractor<'a> {
    global: &'a VarEnv,
    lhs: &'a Var,
}

impl<'a> DepExtractor<'a> {
    pub fn new(global: &'a VarEnv, lhs: &'a Var) -> Self {
        DepExtractor { global, lhs }
    }

    pub fn extract<T>(&self, t: &mut T, forbid_lifted: bool) -> Result<Dependency>
    where
        T: DepsTrailer,
    {
        let tcx = TyCtx::new(&self.global, &self.lhs, forbid_lifted);
        let tcx_cell = RefCell::new(tcx);
        {
            let tcx_ref = TyCtxRef::new(&tcx_cell);
            t.deps_trailer(&tcx_ref);
        }
        tcx_cell.into_inner().try_get_deps()
    }
}

pub trait DepsTrailer {
    fn deps_trailer(&mut self, context: Context);
}

impl DepsTrailer for Expr {
    fn deps_trailer(&mut self, context: Context) {
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
            Lit(e) => {}

            e => unimplemented!("deps_trailer impl: {:?}", e),
        }
    }
}

impl DepsTrailer for ExprPath {
    fn deps_trailer(&mut self, context: Context) {
        context.insert_variable(&self.path);
    }
}

impl DepsTrailer for ArrowExpr {
    fn deps_trailer(&mut self, context: Context) {
        self.expr.deps_trailer(context)
    }
}
