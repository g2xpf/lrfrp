use super::types::{Dependency, TyCtx, TyCtxRef, VarEnv};
use crate::ast::expressions::{ArrowExpr, Expr, ExprBlock, ExprCall, ExprPath};
use crate::ast::statements::Stmt;
use crate::ast::ItemFn;
use std::cell::RefCell;
use std::ops::DerefMut;
use syn::punctuated::Punctuated;
use syn::Result;

// TODO: simpler implementation

pub type Context<'a, 'b, 'c, 'd> = &'a TyCtxRef<'b, 'c, 'd>;

pub struct DepExtractor<'a> {
    global: &'a VarEnv,
}

impl<'a> DepExtractor<'a> {
    pub fn new(global: &'a VarEnv) -> Self {
        DepExtractor { global }
    }

    pub fn extract<'b, T: DepsTrailer<'b>>(
        &self,
        t: &'b mut T,
        forbid_lifted: bool,
    ) -> Result<Dependency<'b>> {
        let tcx = TyCtx::new(&self.global, forbid_lifted);
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

impl<'a, T> DepsTrailer<'a> for Box<T>
where
    T: DepsTrailer<'a>,
{
    fn deps_trailer(&'a mut self, context: Context<'_, '_, '_, 'a>) {
        self.deref_mut().deps_trailer(context);
    }
}

impl<'a, T, P> DepsTrailer<'a> for Punctuated<T, P>
where
    T: DepsTrailer<'a>,
{
    fn deps_trailer(&'a mut self, context: Context<'_, '_, '_, 'a>) {
        self.iter_mut().for_each(|t| {
            t.deps_trailer(context);
        })
    }
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

            Block(e) => e.deps_trailer(context),

            Call(e) => e.deps_trailer(context),

            e => unimplemented!("deps_trailer impl: {:?}", e),
        }
    }
}

impl<'a> DepsTrailer<'a> for ExprCall {
    fn deps_trailer(&'a mut self, context: Context<'_, '_, '_, 'a>) {
        context.insert_variable(&mut self.func.path);
        self.args.deps_trailer(context);
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

impl<'a> DepsTrailer<'a> for ExprBlock {
    fn deps_trailer(&'a mut self, context: Context<'_, '_, '_, 'a>) {
        context.scoped();
        for stmt in self.stmts.iter_mut() {
            match stmt {
                Stmt::Local(e) => {
                    e.expr.deps_trailer(context);
                    context.insert_local(&mut e.pat);
                }
                Stmt::Expr(e) => e.deps_trailer(context),
            }
        }
    }
}

impl<'a> DepsTrailer<'a> for ItemFn {
    fn deps_trailer(&'a mut self, context: Context<'_, '_, '_, 'a>) {
        context.scoped();
        for input in self.inputs.iter_mut() {
            context.insert_local(&mut input.pat);
        }
        self.expr.deps_trailer(context);
    }
}
