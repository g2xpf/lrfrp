use super::deps_trailer::DepsTrailer;
use super::error::UndefinedVariableError;
use crate::ast::expressions::{Expr, ExprPath};

use std::cell::RefCell;
use std::collections::HashMap;

use crate::ast::{
    patterns::{Pat, PatIdent},
    FrpStmtArrow, FrpStmtDependency, ItemFrpStmt,
};

use crate::ast::types;
use syn::Result;

#[derive(Debug)]
pub enum Type {
    Type(MaybeType),
    Primitive(TypePrimitive),
}

impl Type {
    pub fn unresolved() -> Self {
        Type::Type(MaybeType::Unresolved)
    }

    pub fn unresolved_cell() -> Self {
        Type::Primitive(TypePrimitive::Cell(MaybeType::Unresolved))
    }

    pub fn from_ast_type(ty: &types::Type) -> Self {
        Type::Type(MaybeType::Resolved(Box::new(ty.clone())))
    }

    pub fn from_input(ty: &types::Type) -> Self {
        Type::Primitive(TypePrimitive::Input(MaybeType::Resolved(Box::new(
            ty.clone(),
        ))))
    }

    pub fn from_output(ty: &types::Type) -> Self {
        Type::Primitive(TypePrimitive::Output(MaybeType::Resolved(Box::new(
            ty.clone(),
        ))))
    }

    pub fn from_args(ty: &types::Type) -> Self {
        Type::Primitive(TypePrimitive::Args(MaybeType::Resolved(Box::new(
            ty.clone(),
        ))))
    }
}

#[derive(Debug)]
pub enum TypePrimitive {
    Cell(MaybeType),
    Input(MaybeType),
    Output(MaybeType),
    Args(MaybeType),
}

#[derive(Debug)]
pub enum MaybeType {
    Unresolved,
    Resolved(Box<types::Type>),
}

pub type Var = String;
pub type Dependency = (Var, Vec<Var>);
pub type VarEnv = HashMap<Var, Type>;

pub struct TyCtx<'a> {
    global: &'a VarEnv,

    scope: usize,
    local: Vec<VarEnv>,
    deps: Dependency,
    forbid_signal: bool,
}

impl<'a> TyCtx<'a> {
    fn new(global: &'a VarEnv, lhs: &Var, forbid_signal: bool) -> Self {
        let mut ty_ctx = TyCtx {
            global,
            scope: 0,
            local: vec![],
            deps: (lhs.clone(), Vec::new()),
            forbid_signal,
        };
        ty_ctx.scoped();
        ty_ctx
    }

    fn forbid_signal(&self) -> bool {
        self.forbid_signal
    }

    fn set_local(&mut self, var: &Var) {
        self.local[self.scope - 1].insert(var.clone(), Type::unresolved());
    }

    fn try_register(&mut self, expr_path: &ExprPath) -> Result<()> {
        // search local scope
        let ident = expr_path.get_ident();
        let key = ident.to_string();
        for scope in (0..self.scope).rev() {
            if self.local[scope].contains_key(&key) {
                return Ok(());
            }
        }

        // search global scope
        if self.global.contains_key(&key) {
            self.deps.1.push(key);
            Ok(())
        } else {
            Err(UndefinedVariableError::new(ident).into())
        }
    }

    fn scoped(&mut self) {
        self.scope += 1;
        self.local.push(VarEnv::new());
    }

    fn unscoped(&mut self) {
        self.scope -= 1;
        self.local.pop();
    }
}

pub struct TyCtxRef<'a, 'b: 'a>(&'a RefCell<TyCtx<'b>>);

impl TyCtxRef<'_, '_> {
    fn scoped(&self) -> Self {
        TyCtxRef(self.0)
    }

    fn set_local(&self, var: &Var) {
        self.0.borrow_mut().set_local(var)
    }

    fn try_register(&self, expr_path: &ExprPath) -> Result<()> {
        self.0.borrow_mut().try_register(expr_path)
    }

    fn forbid_signal(&self) -> bool {
        self.0.borrow().forbid_signal()
    }

    fn run(global: &mut VarEnv, mut frp_stmts: Vec<ItemFrpStmt>) -> Result<Vec<Dependency>> {
        frp_stmts
            .into_iter()
            .try_fold(vec![], |mut acc, frp_stmt| match frp_stmt {
                ItemFrpStmt::Dependency(FrpStmtDependency { pat, mut expr, .. }) => {
                    let lhs = match pat {
                        Pat::Wild(_) => return Ok(acc),
                        Pat::Ident(PatIdent { ident, .. }) => ident.to_string(),
                        _ => unimplemented!(),
                    };
                    let tcx = TyCtx::new(global, &lhs, false);
                    let tcx_cell = RefCell::new(tcx);
                    let tcx_ref = TyCtxRef(&tcx_cell);
                    acc.push(expr.deps_trailer(&tcx_ref)?);
                    Ok(acc)
                }
                ItemFrpStmt::Arrow(FrpStmtArrow {
                    pat,
                    mut arrow_expr,
                    mut expr,
                    ..
                }) => {
                    let lhs = match pat {
                        Pat::Wild(_) => return Ok(acc),
                        Pat::Ident(PatIdent { ident, .. }) => ident.to_string(),
                        _ => unimplemented!(),
                    };
                    let tcx = TyCtx::new(global, &lhs, true);
                    let tcx_cell = RefCell::new(tcx);
                    let tcx_ref = TyCtxRef(&tcx_cell);
                    arrow_expr.deps_trailer(&tcx_ref)?;

                    let tcx = TyCtx::new(global, &lhs, false);
                    let tcx_cell = RefCell::new(tcx);
                    let tcx_ref = TyCtxRef(&tcx_cell);
                    acc.push(expr.deps_trailer(&tcx_ref)?);
                    Ok(acc)
                }
            })
    }
}

impl Drop for TyCtxRef<'_, '_> {
    fn drop(&mut self) {
        self.0.borrow_mut().unscoped();
    }
}

pub struct DepsChecker {}
