use super::deps_trailer::DepExtractor;
use super::error::{MultipleDefinitionError, NotCalculatedError};
use super::tsort;
use super::types::{Type, Var, VarEnv};

use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};

use crate::ast::patterns::{Pat, PatIdent};
use crate::ast::{Field, FrpStmtArrow, FrpStmtDependency, ItemArgs, ItemFrpStmt, ItemIn, ItemOut};
use syn::Result;

use std::collections::hash_map::Entry;

#[derive(Debug)]
pub struct OrderedStmts {
    dependencies: Vec<FrpStmtDependency>,
    arrows: Vec<FrpStmtArrow>,
}

pub fn deps_check(
    input: &ItemIn,
    output: &ItemOut,
    args: &Option<ItemArgs>,
    mut frp_stmts: Vec<ItemFrpStmt>,
) -> Result<OrderedStmts> {
    // collect identifiers of global let-bindings
    let calculation_order = {
        let global = collect_global_idents(&input, &output, &args, &frp_stmts)?;

        let deps = extract_deps(&global, &mut frp_stmts)?;
        // eprintln!("{:#?}", deps);
        tsort::tsort(&deps)?
    };

    // eprintln!("{:?}", calculation_order);

    Ok(generate_ordered_stmts(frp_stmts, calculation_order))
}

fn generate_ordered_stmts(
    frp_stmts: Vec<ItemFrpStmt>,
    calculation_order: Vec<String>,
) -> OrderedStmts {
    let mut stmt_map = HashMap::new();
    let mut dependencies = vec![];
    let mut arrows = vec![];

    frp_stmts.into_iter().for_each(|frp_stmt| match frp_stmt {
        ItemFrpStmt::Dependency(dep) => {
            let lhs: Var = match &dep.pat {
                Pat::Wild(_) => return,
                Pat::Ident(PatIdent { ref ident, .. }) => ident,
                _ => unimplemented!(),
            };
            stmt_map.insert(lhs.to_string(), dep);
        }
        ItemFrpStmt::Arrow(arrow) => {
            arrows.push(arrow);
        }
    });

    calculation_order.iter().for_each(|s| {
        if let Some(frp_stmt) = stmt_map.remove(s) {
            dependencies.push(frp_stmt);
        }
    });

    OrderedStmts {
        dependencies,
        arrows,
    }
}

fn extract_deps<'a, 'b>(
    global: &'a VarEnv,
    frp_stmts: &'b mut Vec<ItemFrpStmt>,
) -> Result<HashMap<Var<'b>, HashSet<Var<'b>>>> {
    frp_stmts
        .iter_mut()
        .try_fold(HashMap::new(), |mut acc, frp_stmt| {
            let e = match frp_stmt {
                ItemFrpStmt::Dependency(FrpStmtDependency {
                    ref pat,
                    ref mut expr,
                    ..
                }) => {
                    let lhs: Var = match pat {
                        Pat::Wild(_) => return Ok(acc),
                        Pat::Ident(PatIdent { ref ident, .. }) => ident,
                        _ => unimplemented!(),
                    };
                    let extractor = DepExtractor::new(global, lhs);
                    let (lhs, dep) = extractor.extract(expr, false)?;
                    acc.insert(lhs, dep);
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
                        Pat::Ident(PatIdent { ident, .. }) => ident,
                        _ => unimplemented!(),
                    };
                    let extractor = DepExtractor::new(global, &lhs);
                    extractor.extract(arrow_expr, true)?;

                    let extractor = DepExtractor::new(global, &lhs);
                    extractor.extract(expr, false)?;
                    Ok(acc)
                }
            };
            e
        })
}

fn collect_global_idents(
    input: &ItemIn,
    output: &ItemOut,
    args: &Option<ItemArgs>,
    frp_stmts: &Vec<ItemFrpStmt>,
) -> Result<VarEnv> {
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
                    match acc.entry(ident.clone()) {
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
                    match acc.entry(ident.clone()) {
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
            match global.entry(ident.clone()) {
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
            match global.entry(ident.clone()) {
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
            match global.entry(ident.clone()) {
                Entry::Vacant(e) => {
                    e.insert(Type::resolved(ty));
                }
                Entry::Occupied(_) => return Err(MultipleDefinitionError::new(ident).into()),
            }
        }
    }

    Ok(global)
}
