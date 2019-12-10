use super::BinOp;
use syn::parse::ParseStream;
use syn::Token;

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub enum Precedence {
    Any,
    Or,
    And,
    Compare,
    BitOr,
    BitXor,
    BitAnd,
    Shift,
    Arithmetic,
    Term,
    Pow,
    Cast,
}

impl Precedence {
    pub fn of(op: &BinOp) -> Self {
        match *op {
            BinOp::Add(_) | BinOp::Sub(_) => Precedence::Arithmetic,
            BinOp::Mul(_) | BinOp::Div(_) | BinOp::Rem(_) => Precedence::Term,
            BinOp::And(_) => Precedence::And,
            BinOp::Or(_) => Precedence::Or,
            BinOp::BitXor(_) => Precedence::BitXor,
            BinOp::BitAnd(_) => Precedence::BitAnd,
            BinOp::BitOr(_) => Precedence::BitOr,
            BinOp::Shl(_) | BinOp::Shr(_) => Precedence::Shift,
            BinOp::Eq(_)
            | BinOp::Lt(_)
            | BinOp::Le(_)
            | BinOp::Ne(_)
            | BinOp::Ge(_)
            | BinOp::Gt(_) => Precedence::Compare,
            BinOp::Pow(_) => Precedence::Pow,
        }
    }

    pub fn peek(input: ParseStream) -> Self {
        if let Ok(op) = input.fork().parse() {
            Precedence::of(&op)
        } else if input.peek(Token![as]) || input.peek(Token![:]) && !input.peek(Token![::]) {
            Precedence::Cast
        } else {
            Precedence::Any
        }
    }
}
