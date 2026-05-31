use diagnostics::CompileResult;
use hir::HirExpr;
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_index_get(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
    ) -> CompileResult<MirOperand> {
        let obj_operand = self.lower_expr(object)?;
        let idx_operand = self.lower_expr(index)?;
        let dest = self.fresh_reg();
        self.emit(MirInstr::CallDirect(
            dest,
            "__bs_index_get".to_string(),
            vec![obj_operand, idx_operand],
        ));
        Ok(MirOperand::Reg(dest))
    }
}
