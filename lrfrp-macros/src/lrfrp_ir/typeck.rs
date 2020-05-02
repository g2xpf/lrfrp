use super::deps_check;
use crate::ast::{Declaration, ItemArgs, ItemIn, ItemOut, Path};
use std::collections::{HashMap, HashSet};
use syn::Ident;

use syn::Result;

pub fn typeck(
    input: &ItemIn,
    output: &ItemOut,
    args: &Option<ItemArgs>,
    declarations: &mut Vec<Declaration>,
    body: &mut deps_check::OrderedStmts,
) {
}

pub type Type = String;
pub type Var<'a> = &'a Ident;
pub type VarEnv<'a> = HashMap<Var<'a>, Type>;
pub type TypeDefinition<'a> = HashMap<Var<'a>, Type>;

pub struct GlobalTypeEnv<'a> {
    types: HashSet<Type>,
    // Type -> Member -> Type
    definitions: TypeDefinition<'a>,
}

impl<'a> GlobalTypeEnv<'a> {
    fn new() -> Self {
        GlobalTypeEnv {
            types: HashSet::new(),
            definitions: HashMap::new(),
        }
    }
}

pub struct LocalTypeEnv<'a> {
    local: Vec<VarEnv<'a>>,
    deps: usize,
}

impl<'a> LocalTypeEnv<'a> {
    fn new() -> Self {
        LocalTypeEnv {
            local: vec![],
            deps: 1,
        }
    }
}

pub struct Typeck<'a> {
    global: GlobalTypeEnv<'a>,
    local: LocalTypeEnv<'a>,
}

impl<'a> Typeck<'a> {
    fn new() -> Self {
        Typeck {
            global: GlobalTypeEnv::new(),
            local: LocalTypeEnv::new(),
        }
    }

    fn initialize(
        mut self,
        input: &ItemIn,
        output: &ItemOut,
        args: &Option<ItemArgs>,
    ) -> Result<Self> {
    }

    fn typeck(
        mut self,
        declarations: &mut Vec<Declaration>,
        body: &mut OrderedStmts,
    ) -> Result<()> {
    }
}
