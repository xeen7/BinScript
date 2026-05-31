use diagnostics::CompileResult;
use hir::{HirExpr, BindingId};
use crate::types::*;
use super::super::LowerCtx;


impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_let(
        &mut self,
        binding: &BindingId,
        name: &str,
        init: &Option<HirExpr>,
    ) -> CompileResult<()> {
        let reg = self.fresh_reg();
        self.bind(*binding, reg);

        if self.capture_cells.contains(binding) {
            self.reg_shapes.insert(reg, "CaptureCell".to_string());
            self.emit(MirInstr::Alloc(reg, "CaptureCell".to_string()));
            let val = match init {
                Some(e) => self.lower_expr(e)?,
                None => MirOperand::ConstUndefined,
            };
            self.emit(MirInstr::StoreField(reg, 0, val.clone()));
            // Track class constructor bindings for static getter/setter interception
            if self.classes.contains_key(name) {
                self.emit(MirInstr::StoreGlobal(format!("__bs_class_val_{}", name), val));
            }
        } else {
            let val = match init {
                Some(e) => self.lower_expr(e)?,
                None => MirOperand::ConstUndefined,
            };
            if let MirOperand::Reg(src_reg) = &val {
                if let Some(shape) = self.reg_shapes.get(&src_reg).cloned() {
                    self.reg_shapes.insert(reg, shape);
                }
            }
            self.emit(MirInstr::Move(reg, val));
            // Track class constructor bindings for static getter/setter interception
            if self.classes.contains_key(name) {
                self.class_constructors.insert(reg, name.to_string());
                self.emit(MirInstr::StoreGlobal(format!("__bs_class_val_{}", name), MirOperand::Reg(reg)));
            }
        }
        Ok(())
    }
}
