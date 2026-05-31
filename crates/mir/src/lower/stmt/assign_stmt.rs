use diagnostics::CompileResult;
use hir::{HirExpr, BindingId};
use crate::types::*;
use super::super::LowerCtx;


impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_assign(
        &mut self,
        target: &BindingId,
        value: &HirExpr,
    ) -> CompileResult<()> {
        let val = self.lower_expr(value)?;
        if let Some(&reg) = self.bindings.get(target) {
            if self.capture_cells.contains(target) {
                self.emit(MirInstr::StoreField(reg, 0, val));
            } else {
                if let MirOperand::Reg(src_reg) = &val {
                    if let Some(shape) = self.reg_shapes.get(&src_reg).cloned() {
                        self.reg_shapes.insert(reg, shape);
                    }
                }
                self.emit(MirInstr::Move(reg, val));
            }
        }
        Ok(())
    }
}
