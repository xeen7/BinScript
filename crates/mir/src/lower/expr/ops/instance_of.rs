use diagnostics::{CompileError, CompileResult};
use hir::HirExpr;
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_instance_of(
        &mut self,
        expr: &HirExpr,
        class_name: &str,
    ) -> CompileResult<MirOperand> {
        let ev = self.lower_expr(expr)?;
        let builtin_shape = match class_name {
            "Object" => Some(0),
            "String" => Some(1002),
            "Number" => Some(1003),
            "Boolean" => Some(1004),
            "Date" => Some(1005),
            "Map" => Some(1006),
            "Set" => Some(1007),
            "WeakMap" => Some(1008),
            "WeakSet" => Some(1009),
            "Error" => Some(1010),
            "RegExp" => Some(1011),
            _ => None,
        };
        
        let shape_id = if let Some(id) = builtin_shape {
            id
        } else {
            *self.class_shapes.get(class_name).ok_or_else(|| {
                CompileError::Lowering {
                    message: format!("Class '{}' in instanceof check is not defined", class_name),
                }
            })?
        };
        let dest = self.fresh_reg();
        self.emit(MirInstr::CallDirect(
            dest,
            "__bs_instanceof".to_string(),
            vec![ev, MirOperand::ConstNum(shape_id as f64)],
        ));
        Ok(MirOperand::Reg(dest))
    }
}
