use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::LowerCtx;


impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_closure(
        &mut self,
        func_id: &hir::FuncId,
        captures: &[hir::BindingId],
    ) -> CompileResult<MirOperand> {
        let mut mir_caps = Vec::new();
        for &bid in captures {
            if let Some(&reg) = self.bindings.get(&bid) {
                mir_caps.push(MirOperand::Reg(reg));
            } else {
                return Err(CompileError::Lowering {
                    message: format!("Captured binding {} not resolved in bindings when lowering closure for func {:?}", bid, func_id),
                });
            }
        }
        let dest = self.fresh_reg();
        self.emit(MirInstr::AllocClosure(dest, *func_id, mir_caps));
        Ok(MirOperand::Reg(dest))
    }
}
