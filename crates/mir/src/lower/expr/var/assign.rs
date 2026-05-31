use diagnostics::CompileResult;
use hir::HirExpr;
use crate::types::*;
use crate::lower::LowerCtx;

use hir::BindingId;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_assign(
        &mut self,
        target: &BindingId,
        value: &HirExpr,
    ) -> CompileResult<MirOperand> {
        let val = self.lower_expr(value)?;
        // Look up the binding; if it's unknown (e.g. a temp from object literal lowering),
        // auto-allocate a fresh register for it so it can be used later.
        let reg = if let Some(&existing) = self.bindings.get(target) {
            existing
        } else {
            let fresh = self.fresh_reg();
            self.bindings.insert(*target, fresh);
            fresh
        };
        if self.capture_cells.contains(target) {
            self.emit(MirInstr::StoreField(reg, 0, val.clone()));
        } else {
            if let MirOperand::Reg(src_reg) = &val {
                if let Some(shape) = self.reg_shapes.get(&src_reg).cloned() {
                    self.reg_shapes.insert(reg, shape);
                }
            }
            self.emit(MirInstr::Move(reg, val.clone()));
        }
        Ok(val)
    }
}
