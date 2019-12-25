use super::error::{LiftedTypeNotAllowedError, UndefinedVariableError};

use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::ast::types;
use syn::{Ident, Result};

use crate::ast::path::Path;
use crate::ast::patterns::Pat;

#[derive(Clone, Debug)]
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

    pub fn from_args(ty: &types::Type) -> Self {
        Type::Mono(TypeMono::Args(MaybeType::Resolved(Box::new(ty.clone()))))
    }

    pub fn from_local() -> Self {
        Type::Lifted(TypeLifted::Signal(TypeSignal::Local(MaybeType::Unresolved)))
    }
}

#[derive(Clone, Debug)]
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

    fn insert_local(&mut self, pat: &'b mut Pat) {
        use Pat::*;
        let current_local = &mut self.local[self.scope - 1];
        match pat {
            Wild(_) => {}
            Ident(p) => {
                let ident = p.ident.clone();
                current_local.insert(ident, Type::unresolved());
                if let Some((_, pat)) = &mut p.subpat {
                    self.insert_local(pat);
                }
            }
            _ => unimplemented!("insert local for pat"),
        }
    }

    fn insert_variable(&mut self, path: &'b mut Path) {
        // search local scope
        let key = Borrow::<Ident>::borrow(path).clone();
        for scope in (0..self.scope).rev() {
            if self.local[scope].contains_key(&key) {
                return;
            }
        }

        // search global scope
        if let Some(ty) = self.global.get(&key) {
            match ty {
                Type::Lifted(ty) if self.forbid_lifted() => {
                    self.push_error(LiftedTypeNotAllowedError::new(&key, ty))
                }
                _ => {
                    path.typing(ty);
                    self.deps.1.insert(Borrow::<Ident>::borrow(path));
                }
            }
        } else {
            self.push_error(UndefinedVariableError::new(&key))
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

    pub fn scoped(&self) {
        self.0.borrow_mut().scoped();
    }

    pub fn insert_local(&self, pat: &'c mut Pat) {
        self.0.borrow_mut().insert_local(pat)
    }

    pub fn insert_variable(&self, path: &'c mut Path) {
        self.0.borrow_mut().insert_variable(path)
    }

    pub fn forbid_lifted(&self) -> bool {
        self.0.borrow().forbid_lifted()
    }
}

impl Drop for TyCtxRef<'_, '_, '_> {
    fn drop(&mut self) {
        self.0.borrow_mut().unscoped();
    }
}
