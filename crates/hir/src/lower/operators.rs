//! Conversion helpers from SWC AST operators to HIR operators.

use swc_core::ecma::ast::*;
use crate::types::*;

pub(crate) fn conv_bin_op(op: BinaryOp) -> BinOp {
    match op {
        BinaryOp::Exp => BinOp::Exp,
        BinaryOp::NullishCoalescing => BinOp::NullishCoalescing,
        BinaryOp::In => BinOp::In,
        BinaryOp::Add => BinOp::Add,
        BinaryOp::Sub => BinOp::Sub,
        BinaryOp::Mul => BinOp::Mul,
        BinaryOp::Div => BinOp::Div,
        BinaryOp::Mod => BinOp::Mod,
        BinaryOp::EqEq => BinOp::Eq,
        BinaryOp::NotEq => BinOp::Ne,
        BinaryOp::EqEqEq => BinOp::StrictEq,
        BinaryOp::NotEqEq => BinOp::StrictNe,
        BinaryOp::Lt => BinOp::Lt,
        BinaryOp::LtEq => BinOp::Le,
        BinaryOp::Gt => BinOp::Gt,
        BinaryOp::GtEq => BinOp::Ge,
        BinaryOp::LogicalAnd => BinOp::And,
        BinaryOp::LogicalOr => BinOp::Or,
        BinaryOp::BitAnd => BinOp::BitAnd,
        BinaryOp::BitOr => BinOp::BitOr,
        BinaryOp::BitXor => BinOp::BitXor,
        BinaryOp::LShift => BinOp::Shl,
        BinaryOp::RShift => BinOp::Shr,
        BinaryOp::ZeroFillRShift => BinOp::UShr,
        _ => BinOp::Add, // fallback for unsupported ops
    }
}

pub(crate) fn conv_unary_op(op: swc_core::ecma::ast::UnaryOp) -> crate::types::UnaryOp {
    match op {
        swc_core::ecma::ast::UnaryOp::Plus => crate::types::UnaryOp::Plus,
        swc_core::ecma::ast::UnaryOp::Minus => crate::types::UnaryOp::Neg,
        swc_core::ecma::ast::UnaryOp::Bang => crate::types::UnaryOp::Not,
        swc_core::ecma::ast::UnaryOp::Tilde => crate::types::UnaryOp::BitNot,
        swc_core::ecma::ast::UnaryOp::TypeOf => crate::types::UnaryOp::Typeof,
        swc_core::ecma::ast::UnaryOp::Void => crate::types::UnaryOp::Void,
        swc_core::ecma::ast::UnaryOp::Delete => crate::types::UnaryOp::Void, // handled differently
            }
}
