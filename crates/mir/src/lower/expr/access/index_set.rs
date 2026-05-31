use diagnostics::CompileResult;
use hir::HirExpr;
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_index_set(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
        value: &HirExpr,
    ) -> CompileResult<MirOperand> {
        let obj_operand = self.lower_expr(object)?;
        let idx_operand = self.lower_expr(index)?;
        let val_operand = self.lower_expr(value)?;
        let unused = self.fresh_reg();
        self.emit(MirInstr::CallDirect(
            unused,
            "__bs_index_set".to_string(),
            vec![obj_operand, idx_operand, val_operand.clone()],
        ));
        Ok(val_operand)
    }
}
