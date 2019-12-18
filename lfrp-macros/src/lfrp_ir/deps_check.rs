use super::deps_trailer::DepExtractor;
use super::error::{MultipleDefinitionError, NotCalculatedError};
use super::types::{Dependency, TyCtx, TyCtxRef, Type, VarEnv};

use std::borrow::Borrow;

use crate::ast::patterns::{Pat, PatIdent};
use crate::ast::{
    Field, FrpStmtArrow, FrpStmtDependency, ItemArgs, ItemFrpStmt, ItemIn, ItemMod, ItemOut,
};
use syn::{Ident, Result};

use std::cell::RefCell;
use std::collections::hash_map::Entry;

pub fn deps_check(
    module: &ItemMod,
    input: &ItemIn,
    output: &ItemOut,
    args: &Option<ItemArgs>,
    mut frp_stmts: Vec<ItemFrpStmt>,
) -> Result<Vec<ItemFrpStmt>> {
    // collect identifiers of global let-bindings
    let mut global =
        frp_stmts
            .iter()
            .try_fold::<_, _, Result<_>>(VarEnv::new(), |mut acc, frp_stmt| match frp_stmt {
                ItemFrpStmt::Arrow(ref arrow) => {
                    let ident = match &arrow.pat {
                        Pat::Path(e) => e.borrow(),
                        Pat::Ident(e) => &e.ident,
                        e => unimplemented!("uncovered pattern: {:?}", e),
                    };
                    match acc.entry(ident.to_string()) {
                        Entry::Vacant(e) => {
                            e.insert(Type::from_cell(&arrow.ty));
                        }
                        Entry::Occupied(_) => {
                            return Err(MultipleDefinitionError::new(ident).into())
                        }
                    }
                    Ok(acc)
                }
                ItemFrpStmt::Dependency(ref dependency) => {
                    let ident = match &dependency.pat {
                        Pat::Path(e) => e.borrow(),
                        Pat::Ident(e) => &e.ident,
                        e => unimplemented!("uncovered pattern: {:?}", e),
                    };
                    match acc.entry(ident.to_string()) {
                        Entry::Vacant(e) => {
                            e.insert(Type::from_local());
                        }
                        Entry::Occupied(_) => {
                            return Err(MultipleDefinitionError::new(ident).into())
                        }
                    }
                    Ok(acc)
                }
            })?;

    // ensures all the outputs will be calculated
    output
        .fields
        .iter()
        .try_for_each::<_, Result<_>>(|Field { ident, ty, .. }| {
            match global.entry(ident.to_string()) {
                Entry::Occupied(ref mut e) => {
                    let untyped = e.get_mut();
                    *untyped = Type::from_output(ty);
                    Ok(())
                }
                Entry::Vacant(e) => Err(NotCalculatedError::new(ident).into()),
            }
        })?;

    // prevent multiple definition
    input
        .fields
        .iter()
        .try_for_each::<_, Result<_>>(|Field { ident, ty, .. }| {
            match global.entry(ident.to_string()) {
                Entry::Vacant(e) => {
                    e.insert(Type::from_input(ty));
                    Ok(())
                }
                Entry::Occupied(_) => Err(MultipleDefinitionError::new(ident).into()),
            }
        })?;

    // register args if given
    if let Some(ref args) = args {
        for field in args.fields.iter() {
            let ident = &field.ident;
            let ty = &field.ty;
            match global.entry(ident.to_string()) {
                Entry::Vacant(e) => {
                    e.insert(Type::resolved(ty));
                }
                Entry::Occupied(_) => return Err(MultipleDefinitionError::new(ident).into()),
            }
        }
    }

    eprintln!("{:#?}", global);

    let deps = extract_deps(&global, &mut frp_stmts)?;
    println!("{:?}", deps);

    unimplemented!("deps check")
}

fn extract_deps(global: &VarEnv, frp_stmts: &mut Vec<ItemFrpStmt>) -> Result<Vec<Dependency>> {
    frp_stmts
        .iter_mut()
        .try_fold(vec![], |mut acc, mut frp_stmt| {
            let e = match &mut frp_stmt {
                ItemFrpStmt::Dependency(FrpStmtDependency {
                    ref pat,
                    ref mut expr,
                    ..
                }) => {
                    let lhs = match pat {
                        Pat::Wild(_) => return Ok(acc),
                        Pat::Ident(PatIdent { ident, .. }) => ident.to_string(),
                        _ => unimplemented!(),
                    };
                    let extractor = DepExtractor::new(global, &lhs);
                    let dep = extractor.extract(expr, false)?;
                    acc.push(dep);
                    Ok(acc)
                }
                ItemFrpStmt::Arrow(FrpStmtArrow {
                    pat,
                    ref mut arrow_expr,
                    ref mut expr,
                    ..
                }) => {
                    let lhs = match pat {
                        Pat::Wild(_) => return Ok(acc),
                        Pat::Ident(PatIdent { ident, .. }) => ident.to_string(),
                        _ => unimplemented!(),
                    };
                    let extractor = DepExtractor::new(global, &lhs);
                    extractor.extract(arrow_expr, true)?;

                    let extractor = DepExtractor::new(global, &lhs);
                    let dep = extractor.extract(expr, false)?;
                    acc.push(dep);
                    Ok(acc)
                }
            };
            e
        })
}
