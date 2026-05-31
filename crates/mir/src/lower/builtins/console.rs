use diagnostics::CompileResult;
use crate::lower::builtins::BuiltinFn;
use crate::types::*;
use super::super::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_builtin_console(
        &mut self,
        method: &str,
        mir_args: Vec<MirOperand>,
        dest: MirReg,
    ) -> CompileResult<bool> {
        match method {
            "log" | "error" => {
                self.emit(MirInstr::CallBuiltin(dest, BuiltinFn::ConsoleLog, mir_args));
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
