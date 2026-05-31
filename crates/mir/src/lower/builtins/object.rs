use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use super::super::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_builtin_object(
        &mut self,
        method: &str,
        mir_args: Vec<MirOperand>,
        dest: MirReg,
    ) -> CompileResult<bool> {
        match method {
            "keys" | "values" | "entries" | "assign" | "create" | "getPrototypeOf" | "fromEntries" => {
                self.emit(MirInstr::CallDirect(
                    dest,
                    format!("__bs_object_{}", method),
                    mir_args,
                ));
                Ok(true)
            }
            _ => Err(CompileError::Lowering {
                message: format!("Object.{}() not supported", method),
            }),
        }
    }
}
