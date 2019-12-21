use super::deps_trailer::DepExtractor;
use super::error::{MultipleDefinitionError, NotCalculatedError};
use super::tsort;
use super::types::{Type, Var, VarEnv};

use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};

use crate::ast::{Field, FrpStmtArrow, FrpStmtDependency, ItemArgs, ItemFrpStmt, ItemIn, ItemOut};
use syn::{Ident, Result};

use proc_macro2::TokenStream;

use quote::{quote, ToTokens};

use std::collections::hash_map::Entry;

#[derive(Debug)]
pub struct OrderedStmts {
    dependencies: Vec<FrpStmtDependency>,
    arrows: HashMap<String, FrpStmtArrow>,
}

impl OrderedStmts {
    pub fn cell_definition(&self) -> TokenStream {
        let mut fields = TokenStream::new();
        for (_, arrow) in self.arrows.iter() {
            let ident: &Ident = &arrow.path.borrow();
            let colon_token = &arrow.colon_token;
            let ty = &arrow.ty;
            let field = quote! {
                #ident #colon_token #ty,
            };
            fields.extend(field);
        }

        quote! {
            #[derive(Clone, Default)]
            struct Cell {
                #fields
            }
        }
    }

    pub fn calculations(&self) -> TokenStream {
        let mut calculations = TokenStream::new();
        self.dependencies.iter().for_each(|dep| {
            dep.to_tokens(&mut calculations);
        });
        calculations
    }

    pub fn cell_updates(&self) -> TokenStream {
        let mut cell_updates = TokenStream::new();
        for (_, arrow) in self.arrows.iter() {
            let path = &arrow.path;
            let expr = &arrow.expr;
            cell_updates.extend(quote! {
                #path = #expr;
            });
        }

        cell_updates
    }

    pub fn cell_initializations(&self) -> TokenStream {
        let mut cell_initializations = TokenStream::new();
        for (_, arrow) in self.arrows.iter() {
            let path = &arrow.path;
            let expr = &arrow.arrow_expr.expr;
            cell_initializations.extend(quote! {
                #path = #expr;
            });
        }
        cell_initializations
    }
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
        tsort::tsort(&deps)?
    };

    Ok(generate_ordered_stmts(frp_stmts, calculation_order))
}

fn generate_ordered_stmts(
    frp_stmts: Vec<ItemFrpStmt>,
    calculation_order: Vec<String>,
) -> OrderedStmts {
    let mut stmt_map = HashMap::new();
    let mut dependencies = vec![];
    let mut arrows = HashMap::new();

    frp_stmts.into_iter().for_each(|frp_stmt| match frp_stmt {
        ItemFrpStmt::Dependency(dep) => {
            let ident: &Ident = &dep.path.borrow();
            stmt_map.insert(ident.to_string(), dep);
        }
        ItemFrpStmt::Arrow(arrow) => {
            let ident: &Ident = arrow.path.borrow();
            arrows.insert(ident.to_string(), arrow);
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
                    ref mut path,
                    ref mut expr,
                    ..
                }) => {
                    let ty = global.get(Borrow::<Ident>::borrow(path));
                    if let Some(ty) = ty {
                        path.typing(ty);
                    }
                    let ident = Borrow::<Ident>::borrow(path);
                    let extractor = DepExtractor::new(global, ident);
                    let (lhs, dep) = extractor.extract(expr, false)?;
                    acc.insert(ident, dep);
                    Ok(acc)
                }
                ItemFrpStmt::Arrow(FrpStmtArrow {
                    ref mut path,
                    ty,
                    ref mut arrow_expr,
                    ref mut expr,
                    ..
                }) => {
                    path.typing(&Type::from_cell(ty));
                    let ident: &Ident = Borrow::<Ident>::borrow(path);
                    let extractor = DepExtractor::new(global, ident);
                    extractor.extract(arrow_expr, true)?;

                    let extractor = DepExtractor::new(global, ident);
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
                    let ident: &Ident = arrow.path.borrow();
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
                    let ident: &Ident = dependency.path.borrow();
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
                    e.insert(Type::from_args(ty));
                }
                Entry::Occupied(_) => return Err(MultipleDefinitionError::new(ident).into()),
            }
        }
    }

    Ok(global)
}
