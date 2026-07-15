use diagnostics::CompileResult;
use hir::HirExpr;
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_method_string(
        &mut self,
        obj_operand: MirOperand,
        method: &str,
        args: &[HirExpr],
    ) -> CompileResult<Option<MirOperand>> {
        let expected_args = match method {
            "charAt" | "charCodeAt" | "startsWith" | "endsWith" | "split" | "repeat" | "at" => 1,
            "substring" | "replace" | "padStart" | "padEnd" => 2,
            "trim" | "toUpperCase" | "toLowerCase" => 0,
            _ => return Ok(None),
        };

        let method_idx = self.method_indices.get(method).map(|&i| i as f64).unwrap_or(-1.0);
        let mut mir_args = vec![obj_operand];

        for i in 0..expected_args {
            if i < args.len() {
                mir_args.push(self.lower_expr(&args[i])?);
            } else {
                mir_args.push(MirOperand::ConstUndefined);
            }
        }

        mir_args.push(MirOperand::ConstNum(method_idx));

        let dest = self.fresh_reg();
        self.emit(MirInstr::CallDirect(
            dest,
            format!("__bs_call_{}", method),
            mir_args,
        ));
        Ok(Some(MirOperand::Reg(dest)))
    }
}
