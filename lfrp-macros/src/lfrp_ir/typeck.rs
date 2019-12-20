// use crate::ast::{ItemArgs, ItemFrpStmt, ItemIn, ItemOut};
// use syn::Result;
//
// use super::error::MultipleDefinitionError;
//
// use crate::ast::patterns::Pat;
//
// use std::borrow::Borrow;
// use std::collections::hash_map::Entry;
//
// pub fn typeck(
//     input: &ItemIn,
//     output: &ItemOut,
//     args: &Option<ItemArgs>,
//     frp_stmts: &mut Vec<ItemFrpStmt>,
// ) -> Result<Vec<ItemFrpStmt>> {
//     // collect identifiers of global let-bindings
//     frp_stmts
//         .iter_mut()
//         .try_for_each::<_, Result<_>>(|frp_stmt| match frp_stmt {
//             ItemFrpStmt::Arrow(ref arrow) => {
//                 let ident = match &arrow.pat {
//                     Pat::Path(e) => e.borrow(),
//                     Pat::Ident(e) => &e.ident,
//                     e => unimplemented!("uncovered pattern: {:?}", e),
//                 };
//                 match acc.entry(ident.clone()) {
//                     Entry::Vacant(e) => {
//                         e.insert(Type::from_cell(&arrow.ty));
//                     }
//                     Entry::Occupied(_) => return Err(MultipleDefinitionError::new(ident).into()),
//                 }
//                 Ok(acc)
//             }
//             ItemFrpStmt::Dependency(ref dependency) => {
//                 let ident = match &dependency.pat {
//                     Pat::Path(e) => e.borrow(),
//                     Pat::Ident(e) => &e.ident,
//                     e => unimplemented!("uncovered pattern: {:?}", e),
//                 };
//                 match acc.entry(ident.clone()) {
//                     Entry::Vacant(e) => {
//                         e.insert(Type::from_local());
//                     }
//                     Entry::Occupied(_) => return Err(MultipleDefinitionError::new(ident).into()),
//                 }
//                 Ok(acc)
//             }
//         })?;
//
//     // ensures all the outputs will be calculated
//     output
//         .fields
//         .iter()
//         .try_for_each::<_, Result<_>>(|Field { ident, ty, .. }| {
//             match global.entry(ident.clone()) {
//                 Entry::Occupied(ref mut e) => {
//                     let untyped = e.get_mut();
//                     *untyped = Type::from_output(ty);
//                     Ok(())
//                 }
//                 Entry::Vacant(e) => Err(NotCalculatedError::new(ident).into()),
//             }
//         })?;
//
//     // prevent multiple definition
//     input
//         .fields
//         .iter()
//         .try_for_each::<_, Result<_>>(|Field { ident, ty, .. }| {
//             match global.entry(ident.clone()) {
//                 Entry::Vacant(e) => {
//                     e.insert(Type::from_input(ty));
//                     Ok(())
//                 }
//                 Entry::Occupied(_) => Err(MultipleDefinitionError::new(ident).into()),
//             }
//         })?;
//
//     // register args if given
//     if let Some(ref args) = args {
//         for field in args.fields.iter() {
//             let ident = &field.ident;
//             let ty = &field.ty;
//             match global.entry(ident.clone()) {
//                 Entry::Vacant(e) => {
//                     e.insert(Type::resolved(ty));
//                 }
//                 Entry::Occupied(_) => return Err(MultipleDefinitionError::new(ident).into()),
//             }
//         }
//     }
// }
