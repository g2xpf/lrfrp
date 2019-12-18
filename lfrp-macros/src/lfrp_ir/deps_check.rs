use super::error::MultipleDefinitionError;
use super::types::{Type, VarEnv};

use crate::ast::{
    FrpStmtArrow, FrpStmtDependency, ItemArgs, ItemFrpStmt, ItemIn, ItemMod, ItemOut,
};
use syn::{Ident, Result};

use std::collections::hash_map::Entry;

pub fn deps_check(
    module: &ItemMod,
    input: &ItemIn,
    output: &ItemOut,
    args: &Option<ItemArgs>,
    arrows: Vec<ItemFrpStmt>,
) -> Result<Vec<ItemFrpStmt>> {
    let mut global = VarEnv::new();

    // register input
    for field in input.fields.iter() {
        let ident = &field.ident;
        let ty = &field.ty;
        match global.entry(ident.to_string()) {
            Entry::Vacant(e) => {
                e.insert(Type::from_input(ty));
            }
            Entry::Occupied(_) => return Err(MultipleDefinitionError::new(ident).into()),
        }
    }

    for field in output.fields.iter() {
        let ident = &field.ident;
        let ty = &field.ty;
        match global.entry(ident.to_string()) {
            Entry::Vacant(e) => {
                e.insert(Type::from_output(ty));
            }
            Entry::Occupied(_) => return Err(MultipleDefinitionError::new(ident).into()),
        }
    }

    if let Some(ref args) = args {
        for field in args.fields.iter() {
            let ident = &field.ident;
            let ty = &field.ty;
            match global.entry(ident.to_string()) {
                Entry::Vacant(e) => {
                    e.insert(Type::from_output(ty));
                }
                Entry::Occupied(_) => return Err(MultipleDefinitionError::new(ident).into()),
            }
        }
    }

    unimplemented!()
}
