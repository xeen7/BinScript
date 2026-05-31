use diagnostics::CompileResult;
use hir::{HirExpr, UnaryOp as HUnaryOp};
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_unary_op(
        &mut self,
        op: &HUnaryOp,
        arg: &HirExpr,
    ) -> CompileResult<MirOperand> {
        let v = self.lower_expr(arg)?;
        let dest = self.fresh_reg();
        match op {
            HUnaryOp::Plus => self.emit(MirInstr::Plus(dest, v)),
            HUnaryOp::Neg => self.emit(MirInstr::Neg(dest, v)),
            HUnaryOp::Not => self.emit(MirInstr::Not(dest, v)),
            HUnaryOp::Typeof => self.emit(MirInstr::CallDirect(dest, "__bs_typeof".to_string(), vec![v])),
            HUnaryOp::BitNot => self.emit(MirInstr::BitNot(dest, v)),
            HUnaryOp::Void => self.emit(MirInstr::Move(dest, MirOperand::ConstUndefined)),
        }
        Ok(MirOperand::Reg(dest))
    }
}
