//! Conversion helpers from OXC AST operators to HIR operators.

use oxc::ast::ast::*;
use crate::types::*;

pub(crate) fn conv_bin_op(op: BinaryOperator) -> BinOp {
    match op {
        BinaryOperator::Equality => BinOp::Eq,
        BinaryOperator::Inequality => BinOp::Ne,
        BinaryOperator::StrictEquality => BinOp::StrictEq,
        BinaryOperator::StrictInequality => BinOp::StrictNe,
        BinaryOperator::LessThan => BinOp::Lt,
        BinaryOperator::LessEqualThan => BinOp::Le,
        BinaryOperator::GreaterThan => BinOp::Gt,
        BinaryOperator::GreaterEqualThan => BinOp::Ge,
        BinaryOperator::ShiftLeft => BinOp::Shl,
        BinaryOperator::ShiftRight => BinOp::Shr,
        BinaryOperator::ShiftRightZeroFill => BinOp::UShr,
        BinaryOperator::Addition => BinOp::Add,
        BinaryOperator::Subtraction => BinOp::Sub,
        BinaryOperator::Multiplication => BinOp::Mul,
        BinaryOperator::Division => BinOp::Div,
        BinaryOperator::Remainder => BinOp::Mod,
        BinaryOperator::BitwiseOR => BinOp::BitOr,
        BinaryOperator::BitwiseXOR => BinOp::BitXor,
        BinaryOperator::BitwiseAnd => BinOp::BitAnd,
        BinaryOperator::In => BinOp::In,
        BinaryOperator::Instanceof => BinOp::In, // TODO: Instanceof
        BinaryOperator::Exponential => BinOp::Exp,
    }
}

pub(crate) fn conv_logical_op(op: LogicalOperator) -> BinOp {
    match op {
        LogicalOperator::Or => BinOp::Or,
        LogicalOperator::And => BinOp::And,
        LogicalOperator::Coalesce => BinOp::NullishCoalescing,
    }
}

pub(crate) fn conv_unary_op(op: UnaryOperator) -> crate::types::UnaryOp {
    match op {
        UnaryOperator::UnaryPlus => crate::types::UnaryOp::Plus,
        UnaryOperator::UnaryNegation => crate::types::UnaryOp::Neg,
        UnaryOperator::LogicalNot => crate::types::UnaryOp::Not,
        UnaryOperator::BitwiseNot => crate::types::UnaryOp::BitNot,
        UnaryOperator::Typeof => crate::types::UnaryOp::Typeof,
        UnaryOperator::Void => crate::types::UnaryOp::Void,
        UnaryOperator::Delete => crate::types::UnaryOp::Void, // handled differently
    }
}
