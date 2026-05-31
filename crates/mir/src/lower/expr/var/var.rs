use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::LowerCtx;

use hir::BindingId;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_var(&mut self, bid: &BindingId) -> CompileResult<MirOperand> {
        if let Some(&reg) = self.bindings.get(bid) {
            if self.capture_cells.contains(bid) {
                let dest = self.fresh_reg();
                self.emit(MirInstr::LoadField(dest, reg, 0));
                Ok(MirOperand::Reg(dest))
            } else {
                Ok(MirOperand::Reg(reg))
            }
        } else {
            Err(CompileError::Lowering {
                message: format!("Unresolved binding {}", bid),
            })
        }
    }
}
