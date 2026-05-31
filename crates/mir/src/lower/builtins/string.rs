use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use super::super::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_builtin_string(
        &mut self,
        method: &str,
        mir_args: Vec<MirOperand>,
        dest: MirReg,
    ) -> CompileResult<bool> {
        match method {
            "fromCharCode" | "fromCodePoint" => {
                self.emit(MirInstr::CallDirect(
                    dest,
                    format!("__bs_string_{}", method),
                    mir_args,
                ));
                Ok(true)
            }
            _ => Err(CompileError::Lowering {
                message: format!("String.{}() not supported", method),
            }),
        }
    }
}
