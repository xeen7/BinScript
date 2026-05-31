use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_global_ref(&mut self, name: &str) -> CompileResult<MirOperand> {
        if name.starts_with("__bs_class_val_") {
            let dest = self.fresh_reg();
            let class_name = name["__bs_class_val_".len()..].to_string();
            self.emit(MirInstr::LoadGlobal(dest, name.to_string()));
            // Track as class constructor for static getter/setter resolution
            if self.classes.contains_key(&class_name) {
                self.class_constructors.insert(dest, class_name);
            }
            return Ok(MirOperand::Reg(dest));
        }
        match name {
            "NaN" => Ok(MirOperand::ConstNum(std::f64::NAN)),
            "Infinity" => Ok(MirOperand::ConstNum(std::f64::INFINITY)),
            "undefined" => Ok(MirOperand::ConstUndefined),
            "globalThis" => {
                let dest = self.fresh_reg();
                self.emit(MirInstr::CallDirect(dest, "__bs_get_globalThis".to_string(), vec![]));
                Ok(MirOperand::Reg(dest))
            }
            "Symbol" => {
                let dest = self.fresh_reg();
                self.emit(MirInstr::CallDirect(dest, "__bs_get_Symbol_global".to_string(), vec![]));
                Ok(MirOperand::Reg(dest))
            }
            _ => Ok(MirOperand::ConstUndefined),
        }
    }
}
