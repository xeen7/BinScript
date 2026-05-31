use diagnostics::CompileResult;
use hir::HirExpr;
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_delete_prop(
        &mut self,
        object: &HirExpr,
        property: &HirExpr,
    ) -> CompileResult<MirOperand> {
        let obj = self.lower_expr(object)?;
        let prop = self.lower_expr(property)?;
        let dest = self.fresh_reg();
        self.emit(MirInstr::DeleteProp(dest, obj, prop));
        Ok(MirOperand::Reg(dest))
    }
}
