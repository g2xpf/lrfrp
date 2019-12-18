use super::error::{LiftedTypeNotAllowedError, UndefinedVariableError};
use crate::ast::expressions::{Expr, ExprPath};

use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::ast::{patterns::Pat, FrpStmtArrow, FrpStmtDependency, ItemFrpStmt};

use crate::ast::types;
use syn::{Ident, Result};

#[derive(Debug)]
pub enum Type {
    Type(MaybeType),
    Lifted(TypeLifted),
}

impl Type {
    pub fn unresolved() -> Self {
        Type::Type(MaybeType::Unresolved)
    }

    pub fn unresolved_cell() -> Self {
        Type::Lifted(TypeLifted::Cell(MaybeType::Unresolved))
    }

    pub fn resolved(ty: &types::Type) -> Self {
        Type::Type(MaybeType::Resolved(Box::new(ty.clone())))
    }

    pub fn from_cell(ty: &types::Type) -> Self {
        Type::Lifted(TypeLifted::Cell(MaybeType::Resolved(Box::new(ty.clone()))))
    }

    pub fn from_input(ty: &types::Type) -> Self {
        Type::Lifted(TypeLifted::Signal(TypeSignal::Input(MaybeType::Resolved(
            Box::new(ty.clone()),
        ))))
    }

    pub fn from_output(ty: &types::Type) -> Self {
        Type::Lifted(TypeLifted::Signal(TypeSignal::Output(MaybeType::Resolved(
            Box::new(ty.clone()),
        ))))
    }

    pub fn from_local() -> Self {
        Type::Lifted(TypeLifted::Signal(TypeSignal::Local(MaybeType::Unresolved)))
    }
}

#[derive(Debug)]
pub enum TypeLifted {
    Cell(MaybeType),
    Signal(TypeSignal),
}

impl fmt::Display for TypeLifted {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TypeLifted::*;
        match self {
            Cell(ref ty) => write!(f, "Cell<{}>", ty),
            Signal(ref ty) => write!(f, "Signal<{}>", ty),
        }
    }
}

#[derive(Debug)]
pub enum TypeSignal {
    Local(MaybeType),
    Input(MaybeType),
    Output(MaybeType),
}

impl fmt::Display for TypeSignal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TypeSignal::*;
        match self {
            Local(ref ty) => write!(f, "{}", ty),
            Input(ref ty) => write!(f, "{}", ty),
            Output(ref ty) => write!(f, "{}", ty),
        }
    }
}

#[derive(Debug)]
pub enum MaybeType {
    Unresolved,
    Resolved(Box<types::Type>),
}

impl fmt::Display for MaybeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use MaybeType::*;
        match self {
            Unresolved => write!(f, "?"),
            Resolved(ref ty) => write!(f, "{}", ty),
        }
    }
}

pub type Var = String;
pub type Dependency = (Var, HashSet<Var>);
pub type VarEnv = HashMap<Var, Type>;

pub struct TyCtx<'a> {
    global: &'a VarEnv,
    scope: usize,
    local: Vec<VarEnv>,
    deps: Dependency,
    errors: Vec<syn::Error>,
    forbid_lifted: bool,
}

impl<'a> TyCtx<'a> {
    pub fn new(global: &'a VarEnv, lhs: &Var, forbid_lifted: bool) -> Self {
        let mut ty_ctx = TyCtx {
            global,
            scope: 0,
            local: vec![],
            deps: (lhs.clone(), HashSet::new()),
            errors: vec![],
            forbid_lifted,
        };
        ty_ctx.scoped();
        ty_ctx
    }

    pub fn try_get_deps(self) -> Result<Dependency> {
        if !self.errors.is_empty() {
            let mut iter = self.errors.into_iter();
            let head = iter.next().unwrap();
            Err(iter.fold(head, |mut acc, error| {
                acc.combine(error);
                acc
            }))
        } else {
            Ok(self.deps)
        }
    }

    fn forbid_lifted(&self) -> bool {
        self.forbid_lifted
    }

    fn insert_local(&mut self, var: &Var) {
        self.local[self.scope - 1].insert(var.clone(), Type::unresolved());
    }

    fn insert_variable<P>(&mut self, expr_path: &P)
    where
        P: Borrow<Ident>,
    {
        // search local scope
        let ident = expr_path.borrow();
        let key = ident.to_string();
        for scope in (0..self.scope).rev() {
            if self.local[scope].contains_key(&key) {
                return;
            }
        }

        // search global scope
        if let Some(ty) = self.global.get(&key) {
            match ty {
                Type::Lifted(ty) if self.forbid_lifted() => {
                    self.push_error(LiftedTypeNotAllowedError::new(ident, ty))
                }
                _ => {
                    self.deps.1.insert(key);
                }
            }
        } else {
            self.push_error(UndefinedVariableError::new(ident))
        }
    }

    fn push_error<E>(&mut self, e: E)
    where
        E: Into<syn::Error>,
    {
        self.errors.push(e.into())
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

impl<'a, 'b> TyCtxRef<'a, 'b> {
    pub fn new(tcx: &'a RefCell<TyCtx<'b>>) -> Self {
        TyCtxRef(tcx)
    }

    pub fn insert_local(&self, var: &Var) {
        self.0.borrow_mut().insert_local(var)
    }

    pub fn insert_variable<P>(&self, expr_path: &P)
    where
        P: Borrow<Ident>,
    {
        self.0.borrow_mut().insert_variable(expr_path)
    }

    pub fn forbid_lifted(&self) -> bool {
        self.0.borrow().forbid_lifted()
    }

    pub fn push_error<E>(&mut self, e: E)
    where
        E: Into<syn::Error>,
    {
        self.0.borrow_mut().push_error(e)
    }
}

impl Drop for TyCtxRef<'_, '_> {
    fn drop(&mut self) {
        self.0.borrow_mut().unscoped();
    }
}
