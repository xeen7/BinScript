use diagnostics::CompileResult;
use hir::HirExpr;
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_ternary(
        &mut self,
        cond: &HirExpr,
        then_expr: &HirExpr,
        else_expr: &HirExpr,
    ) -> CompileResult<MirOperand> {
        let cv = self.lower_expr(cond)?;
        let dest = self.fresh_reg();
        let then_bb = self.fresh_block();
        let else_bb = self.fresh_block();
        let merge_bb = self.fresh_block();

        self.emit(MirInstr::Branch(cv, then_bb, else_bb));

        self.switch_to(then_bb);
        let tv = self.lower_expr(then_expr)?;
        self.emit(MirInstr::Move(dest, tv));
        self.emit(MirInstr::Jump(merge_bb));

        self.switch_to(else_bb);
        let ev = self.lower_expr(else_expr)?;
        self.emit(MirInstr::Move(dest, ev));
        self.emit(MirInstr::Jump(merge_bb));

        self.switch_to(merge_bb);
        Ok(MirOperand::Reg(dest))
    }
}
