use diagnostics::CompileResult;
use hir::{HirExpr, BinOp as HBinOp};
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_bin_op(
        &mut self,
        op: &HBinOp,
        left: &HirExpr,
        right: &HirExpr,
    ) -> CompileResult<MirOperand> {
        if matches!(op, HBinOp::And | HBinOp::Or | HBinOp::NullishCoalescing) {
            let l = self.lower_expr(left)?;
            let dest = self.fresh_reg();
            let eval_r_bb = self.fresh_block();
            let merge_bb = self.fresh_block();

            self.emit(MirInstr::Move(dest, l.clone()));
            if let HBinOp::And = op {
                // AND: if left is true, evaluate right; else keep left (falsy)
                self.emit(MirInstr::Branch(l, eval_r_bb, merge_bb));
            } else if let HBinOp::Or = op {
                // OR: if left is true, keep left (truthy); else evaluate right
                self.emit(MirInstr::Branch(l, merge_bb, eval_r_bb));
            } else {
                // Nullish Coalescing (??): if left is null/undefined, evaluate right; else keep left
                let cond_reg = self.fresh_reg();
                self.emit(MirInstr::CallDirect(cond_reg, "__bs_is_nullish".to_string(), vec![l]));
                self.emit(MirInstr::Branch(MirOperand::Reg(cond_reg), eval_r_bb, merge_bb));
            }

            self.switch_to(eval_r_bb);
            let r = self.lower_expr(right)?;
            self.emit(MirInstr::Move(dest, r));
            self.emit(MirInstr::Jump(merge_bb));

            self.switch_to(merge_bb);
            return Ok(MirOperand::Reg(dest));
        }

        let l = self.lower_expr(left)?;
        let r = self.lower_expr(right)?;
        let dest = self.fresh_reg();
        let instr = match op {
            HBinOp::Add => MirInstr::Add(dest, l, r),
            HBinOp::Sub => MirInstr::Sub(dest, l, r),
            HBinOp::Mul => MirInstr::Mul(dest, l, r),
            HBinOp::Div => MirInstr::Div(dest, l, r),
            HBinOp::Mod => MirInstr::Mod(dest, l, r),
            HBinOp::Exp => MirInstr::Exp(dest, l, r),
            HBinOp::Eq => MirInstr::Eq(dest, l, r),
            HBinOp::Ne => MirInstr::Ne(dest, l, r),
            HBinOp::StrictEq => MirInstr::StrictEq(dest, l, r),
            HBinOp::StrictNe => MirInstr::StrictNe(dest, l, r),
            HBinOp::Lt => MirInstr::Lt(dest, l, r),
            HBinOp::Le => MirInstr::Le(dest, l, r),
            HBinOp::Gt => MirInstr::Gt(dest, l, r),
            HBinOp::Ge => MirInstr::Ge(dest, l, r),
            HBinOp::In => MirInstr::In(dest, l, r),
            HBinOp::BitAnd => MirInstr::BitAnd(dest, l, r),
            HBinOp::BitOr => MirInstr::BitOr(dest, l, r),
            HBinOp::BitXor => MirInstr::BitXor(dest, l, r),
            HBinOp::Shl => MirInstr::Shl(dest, l, r),
            HBinOp::Shr => MirInstr::Shr(dest, l, r),
            HBinOp::UShr => MirInstr::UShr(dest, l, r),
            HBinOp::And | HBinOp::Or | HBinOp::NullishCoalescing => unreachable!(),
        };
        self.emit(instr);
        Ok(MirOperand::Reg(dest))
    }
}
