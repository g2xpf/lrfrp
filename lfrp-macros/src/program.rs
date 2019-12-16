use super::ast;

pub struct Program {
    module: ast::ItemMod,
    input: ast::ItemIn,
    output: ast::ItemOut,
    args: Option<ast::ItemOut>,
}
