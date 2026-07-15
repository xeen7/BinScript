use diagnostics::CompileResult;
use hir::HirExpr;
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_member_get(
        &mut self,
        object: &HirExpr,
        property: &str,
    ) -> CompileResult<MirOperand> {
        let obj_operand = self.lower_expr(object)?;
        let obj_reg = match obj_operand {
            MirOperand::Reg(r) => r,
            _ => {
                let r = self.fresh_reg();
                self.emit(MirInstr::Move(r, obj_operand));
                r
            }
        };

        let dest = self.fresh_reg();
        if let Some(shape) = self.reg_shapes.get(&obj_reg) {
            if self.has_getter(shape, property) {
                let getter_name = format!("__get_{}", property);
                if let Some(&method_idx) = self.method_indices.get(&getter_name) {
                    let mir_args = vec![MirOperand::Reg(obj_reg)];
                    self.emit(MirInstr::CallVTable(dest, obj_reg, method_idx, mir_args));
                    return Ok(MirOperand::Reg(dest));
                }
            }
            if let Some(index) = self.get_field_index(shape, property) {
                self.emit(MirInstr::LoadField(dest, obj_reg, index));
                return Ok(MirOperand::Reg(dest));
            }
        }
        // Check for static getter on class constructor
        if let Some(ctor_class) = self.class_constructors.get(&obj_reg).cloned() {
            if self.has_static_getter(&ctor_class, property) {
                let getter_prop = format!("__get_{}", property);
                let closure_reg = self.fresh_reg();
                self.emit(MirInstr::LoadProp(closure_reg, obj_reg, getter_prop));
                self.emit(MirInstr::CallClosure(dest, closure_reg, vec![MirOperand::Reg(closure_reg), MirOperand::Reg(obj_reg)]));
                return Ok(MirOperand::Reg(dest));
            }
        }

        // Fallback to dynamic property get (for JsonTape and untyped objects)
        self.emit(MirInstr::LoadProp(dest, obj_reg, property.to_string()));
        Ok(MirOperand::Reg(dest))
    }
}
