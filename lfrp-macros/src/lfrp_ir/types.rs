use super::error::UndefinedVariableError;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use crate::ast::types;
use syn::Ident;

pub enum Type {
    Unresolved,
    Cell(types::Type),
    Type(types::Type),
}
pub type Var = Ident;
pub type Dependency = HashMap<Var, Vec<Var>>;
pub type VarEnv = HashMap<Var, Type>;

pub struct TyCtx<'a> {
    global: &'a VarEnv,
    structs: &'a VarEnv,
    enums: &'a VarEnv,

    scope: usize,
    local: Vec<VarEnv>,
    deps: Dependency,
}

struct TyCtxRef<'a, 'b: 'a>(&'a RefCell<TyCtx<'b>>);

impl TyCtxRef<'_, '_> {
    fn scoped(&self) -> Self {
        TyCtxRef(self.0)
    }

    fn set_local(&self, var: &Var) {
        self.0.borrow_mut().set_local(var)
    }

    fn try_register(&self, var: &Var) -> Result<(), Box<syn::Error>> {
        self.0.borrow_mut().try_register(var)
    }

    fn run<T>(global: &mut VarEnv, init_var: &Var, t: &T) -> Result<Dependency, Box<syn::Error>>
    where
        T: VarTrailer,
    {
        let tcx = TyCtx::new(global, init_var);
        let tcx_cell = RefCell::new(tcx);

        let e = {
            let tcx_ref = TyCtxRef(&tcx_cell);
            t.var_trailer(&tcx_ref)
        };

        match e {
            Err(e) => Err(e),
            Ok(_) => Ok(tcx_cell.into_inner().deps),
        }
    }
}

impl Drop for TyCtxRef<'_, '_> {
    fn drop(&mut self) {
        self.0.borrow_mut().unscoped();
    }
}

impl<'a> TyCtx<'a> {
    fn new(global: &'a VarEnv, structs: &'a VarEnv, enums: &'a VarEnv, init_var: &Var) -> Self {
        let mut ty_ctx = TyCtx {
            global,
            structs,
            enums,
            scope: 0,
            local: vec![],
            deps: {
                let mut deps = Dependency::new();
                deps.insert(init_var.clone(), Vec::new());
                deps
            },
        };
        ty_ctx.scoped();
        ty_ctx
    }

    fn set_local(&mut self, var: &Var) {
        self.local[self.scope - 1].insert(var.clone());
    }

    fn try_register(&mut self, var: &Var) -> Result<(), Box<syn::Error>> {
        // search local scope
        for scope in (0..self.scope).rev() {
            if self.local[scope].contains(var) {
                return Ok(());
            }
        }

        // search global scope
        if self.global.contains(var) {
            self.deps.1.push(var.clone());
            Ok(())
        } else {
            Err(Box::new(UndefinedVariableError::generate(var.clone())))
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

pub trait VarTrailer {
    fn var_trailer(&self, varenv: VarEnv) -> Result<(), Box<syn::Error>>;
}

pub struct DepsChecker {}
