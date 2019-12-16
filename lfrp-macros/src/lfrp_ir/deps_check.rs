use crate::ast::{
    FrpStmtArrow, FrpStmtDependency, ItemArgs, ItemFrpStmt, ItemIn, ItemMod, ItemOut,
};
use syn::Result;

struct Env {}

struct DepsChecker {}

pub fn deps_check(
    module: &ItemMod,
    input: &ItemIn,
    output: &ItemOut,
    args: &Option<ItemArgs>,
    deps: Vec<FrpStmtDependency>,
    arrows: Vec<FrpStmtArrow>,
) -> Result<Vec<ItemFrpStmt>> {
    unimplemented!()
}
