use super::error::{LiftedTypeNotAllowedError, UndefinedVariableError};

use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::ast::types;
use syn::{Ident, Result};

#[derive(Debug)]
pub enum Type {
    Mono(TypeMono),
    Lifted(TypeLifted),
}

impl Type {
    pub fn unresolved() -> Self {
        Type::Mono(TypeMono::Type(MaybeType::Unresolved))
    }

    pub fn unresolved_cell() -> Self {
        Type::Lifted(TypeLifted::Cell(MaybeType::Unresolved))
    }

    pub fn resolved(ty: &types::Type) -> Self {
        Type::Mono(TypeMono::Type(MaybeType::Resolved(Box::new(ty.clone()))))
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
pub enum TypeMono {
    Type(MaybeType),
    Args(MaybeType),
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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

pub type Var<'a> = &'a Ident;
pub type Dependency<'a> = (Var<'a>, HashSet<Var<'a>>);
pub type VarEnv = HashMap<Ident, Type>;

pub struct TyCtx<'a, 'b> {
    global: &'a VarEnv,
    scope: usize,
    local: Vec<VarEnv>,
    deps: Dependency<'b>,
    errors: Vec<syn::Error>,
    forbid_lifted: bool,
}

impl<'a, 'b> TyCtx<'a, 'b> {
    pub fn new(global: &'a VarEnv, lhs: Var<'b>, forbid_lifted: bool) -> Self {
        let mut ty_ctx = TyCtx {
            global,
            scope: 0,
            local: vec![],
            deps: (lhs, HashSet::new()),
            errors: vec![],
            forbid_lifted,
        };
        ty_ctx.scoped();
        ty_ctx
    }

    pub fn try_get_deps(self) -> Result<Dependency<'b>> {
        if !self.errors.is_empty() {
            let mut iter = self.errors.into_iter();
            let head = iter.next().unwrap_or_else(|| unreachable!());
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

    fn insert_local(&mut self, var: Var<'b>) {
        self.local[self.scope - 1].insert(var.clone(), Type::unresolved());
    }

    fn insert_variable<P>(&mut self, expr_path: &'b P)
    where
        P: Borrow<Ident>,
    {
        // search local scope
        let key = expr_path.borrow();
        for scope in (0..self.scope).rev() {
            if self.local[scope].contains_key(&key) {
                return;
            }
        }

        // search global scope
        if let Some(ty) = self.global.get(&key) {
            match ty {
                Type::Lifted(ty) if self.forbid_lifted() => {
                    self.push_error(LiftedTypeNotAllowedError::new(key, ty))
                }
                _ => {
                    self.deps.1.insert(key);
                }
            }
        } else {
            self.push_error(UndefinedVariableError::new(key))
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

pub struct TyCtxRef<'a, 'b: 'a, 'c: 'a>(&'a RefCell<TyCtx<'b, 'c>>);

impl<'a, 'b, 'c> TyCtxRef<'a, 'b, 'c> {
    pub fn new(tcx: &'a RefCell<TyCtx<'b, 'c>>) -> Self {
        TyCtxRef(tcx)
    }

    pub fn insert_local(&self, var: Var<'c>) {
        self.0.borrow_mut().insert_local(var)
    }

    pub fn insert_variable<P>(&self, expr_path: &'c P)
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

impl Drop for TyCtxRef<'_, '_, '_> {
    fn drop(&mut self) {
        self.0.borrow_mut().unscoped();
    }
}
