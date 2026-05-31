use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use super::super::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_builtin_math(
        &mut self,
        method: &str,
        mir_args: Vec<MirOperand>,
        dest: MirReg,
    ) -> CompileResult<bool> {
        match method {
            "floor" | "ceil" | "round" | "abs" | "sqrt" | "pow" | "min" | "max"
            | "log" | "log2" | "sin" | "cos" | "tan" | "random" | "trunc" => {
                self.emit(MirInstr::CallDirect(
                    dest,
                    format!("__bs_math_{}", method),
                    mir_args,
                ));
                Ok(true)
            }
            _ => Err(CompileError::Lowering {
                message: format!("Math.{}() not supported", method),
            }),
        }
    }
}
