use super::deps_trailer::DepExtractor;
use super::error::{CellAsOutputError, MultipleDefinitionError, NotCalculatedError};
use super::tsort;
use super::types::{Type, TypeLifted, Var, VarEnv};

use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};

use crate::ast::{
    Field, FrpStmtArrow, FrpStmtArrows, FrpStmtDependency, ItemArgs, ItemDeclaration, ItemFrpStmt,
    ItemIn, ItemOut,
};
use syn::{Ident, Result};

use std::collections::hash_map::Entry;

#[derive(Debug)]
pub struct OrderedStmts {
    pub dependencies: Vec<FrpStmtDependency>,
    pub arrows: FrpStmtArrows,
}

struct VarDependency<'a> {
    pub dependencies: HashMap<Var<'a>, HashSet<Var<'a>>>,
    pub arrows: HashMap<Var<'a>, HashSet<Var<'a>>>,
}

impl VarDependency<'_> {
    fn new() -> Self {
        VarDependency {
            dependencies: HashMap::new(),
            arrows: HashMap::new(),
        }
    }
}

pub fn deps_check(
    input: &ItemIn,
    output: &ItemOut,
    args: &Option<ItemArgs>,
    declarations: &mut [ItemDeclaration],
    mut frp_stmts: Vec<ItemFrpStmt>,
) -> Result<OrderedStmts> {
    // collect identifiers of global let-bindings
    let calculation_order = {
        let global = collect_global_idents(&input, &output, &args, declarations, &frp_stmts)?;

        let deps = extract_deps(&global, declarations, &mut frp_stmts)?;
        let sorted_dependencies = tsort::tsort(&deps.dependencies)?
            .map(|ident| ident.to_string())
            .rev()
            .collect();
        let sorted_arrows = tsort::tsort(&deps.arrows)?
            .map(|ident| ident.to_string())
            .collect();
        (sorted_dependencies, sorted_arrows)
    };

    Ok(generate_ordered_stmts(frp_stmts, calculation_order))
}

fn generate_ordered_stmts(
    frp_stmts: Vec<ItemFrpStmt>,
    calculation_order: (Vec<String>, Vec<String>),
) -> OrderedStmts {
    let mut deps_map = HashMap::new();
    let mut arrows_map = HashMap::new();
    let mut dependencies = vec![];
    let mut arrows = FrpStmtArrows::new();

    frp_stmts.into_iter().for_each(|frp_stmt| match frp_stmt {
        ItemFrpStmt::Dependency(dep) => {
            let ident: &Ident = dep.path.borrow();
            deps_map.insert(ident.to_string(), dep);
        }
        ItemFrpStmt::Arrow(arrow) => {
            let ident: &Ident = arrow.path.borrow();
            arrows_map.insert(ident.to_string(), arrow);
        }
    });

    calculation_order.0.iter().for_each(|s| {
        if let Some(frp_stmt) = deps_map.remove(s) {
            dependencies.push(frp_stmt);
        }
    });

    calculation_order.1.iter().for_each(|s| {
        if let Some(frp_stmt) = arrows_map.remove(s) {
            arrows.push(frp_stmt);
        }
    });

    OrderedStmts {
        dependencies,
        arrows,
    }
}

fn extract_deps<'a, 'b>(
    global: &'a VarEnv,
    declarations: &'b mut [ItemDeclaration],
    frp_stmts: &'b mut Vec<ItemFrpStmt>,
) -> Result<VarDependency<'b>> {
    declarations
        .iter_mut()
        .try_for_each::<_, Result<_>>(|declaration| {
            use ItemDeclaration::*;
            match declaration {
                Struct(_) => unimplemented!("extract_deps case"),
                Enum(_) => unimplemented!("extract_deps case"),
                Fn(e) => {
                    let extractor = DepExtractor::new(global);
                    extractor.extract(e, true)?;
                    Ok(())
                }
            }
        })?;
    frp_stmts
        .iter_mut()
        .try_fold(VarDependency::new(), |mut acc, frp_stmt| match frp_stmt {
            ItemFrpStmt::Dependency(FrpStmtDependency { path, expr, .. }) => {
                let ty = global.get(Borrow::<Ident>::borrow(path));
                if let Some(ty) = ty {
                    path.typing(ty);
                }
                let ident = Borrow::<Ident>::borrow(path);
                let extractor = DepExtractor::new(global);
                let dep = extractor.extract(expr, false)?;
                acc.dependencies.insert(ident, dep);
                Ok(acc)
            }
            ItemFrpStmt::Arrow(FrpStmtArrow {
                path,
                ty,
                arrow_expr,
                expr,
                ..
            }) => {
                path.typing(&Type::from_cell(ty));
                let ident: &Ident = Borrow::<Ident>::borrow(path);
                let extractor = DepExtractor::new(global);
                extractor.extract(arrow_expr, true)?;

                let extractor = DepExtractor::new(global);
                let dep = extractor.extract(expr, false)?;
                acc.arrows.insert(ident, dep);
                Ok(acc)
            }
        })
}

fn collect_global_idents(
    input: &ItemIn,
    output: &ItemOut,
    args: &Option<ItemArgs>,
    declarations: &[ItemDeclaration],
    frp_stmts: &[ItemFrpStmt],
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

    // register declarations
    declarations
        .iter()
        .try_for_each::<_, Result<_>>(|declaration| {
            use ItemDeclaration::*;
            match declaration {
                Struct(_) => unimplemented!("struct pattern"),
                Enum(_) => unimplemented!("enum pattern"),
                Fn(e) => {
                    let ident = &e.ident;
                    let ty = &e.output;
                    match global.entry(ident.clone()) {
                        Entry::Vacant(e) => {
                            e.insert(Type::from_type(ty));
                            Ok(())
                        }
                        Entry::Occupied(_) => Err(MultipleDefinitionError::new(ident).into()),
                    }
                }
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
                    // prevent from using the output variables as Cell
                    if let Type::Lifted(TypeLifted::Cell(_)) = untyped {
                        return Err(CellAsOutputError::new(e.key()).into());
                    }
                    *untyped = Type::from_output(ty);
                    Ok(())
                }
                Entry::Vacant(_) => Err(NotCalculatedError::new(ident).into()),
            }
        })?;

    // prevent from multiple definition
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
