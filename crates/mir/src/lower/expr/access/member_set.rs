use diagnostics::CompileResult;
use hir::HirExpr;
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_member_set(
        &mut self,
        object: &HirExpr,
        property: &str,
        value: &HirExpr,
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

        let val = self.lower_expr(value)?;

        if let Some(shape) = self.reg_shapes.get(&obj_reg) {
            if self.has_setter(shape, property) {
                let setter_name = format!("__set_{}", property);
                if let Some(&method_idx) = self.method_indices.get(&setter_name) {
                    let dest = self.fresh_reg();
                    let mir_args = vec![MirOperand::Reg(obj_reg), val.clone()];
                    self.emit(MirInstr::CallVTable(dest, obj_reg, method_idx, mir_args));
                    return Ok(val);
                }
            }
            if let Some(index) = self.get_field_index(shape, property) {
                self.emit(MirInstr::StoreField(obj_reg, index, val.clone()));
                return Ok(val);
            }
        }

        // Check for static setter on class constructor
        if let Some(ctor_class) = self.class_constructors.get(&obj_reg).cloned() {
            if self.has_static_setter(&ctor_class, property) {
                let setter_prop = format!("__set_{}", property);
                let closure_reg = self.fresh_reg();
                self.emit(MirInstr::LoadProp(closure_reg, obj_reg, setter_prop));
                let dest = self.fresh_reg();
                self.emit(MirInstr::CallClosure(dest, closure_reg, vec![MirOperand::Reg(closure_reg), val.clone()]));
                return Ok(val);
            }
        }

        // Fallback to dynamic property set
        self.emit(MirInstr::StoreProp(obj_reg, property.to_string(), val.clone()));
        Ok(val)
    }
}
