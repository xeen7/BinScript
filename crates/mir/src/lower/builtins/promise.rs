use diagnostics::CompileResult;
use crate::lower::builtins::BuiltinFn;
use crate::types::*;
use super::super::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_builtin_promise(
        &mut self,
        method: &str,
        mir_args: Vec<MirOperand>,
        dest: MirReg,
    ) -> CompileResult<bool> {
        match method {
            "all_2" => {
                self.emit(MirInstr::CallBuiltin(dest, BuiltinFn::PromiseAll2, mir_args));
                Ok(true)
            }
            "race_2" => {
                self.emit(MirInstr::CallBuiltin(dest, BuiltinFn::PromiseRace2, mir_args));
                Ok(true)
            }
            "resolve" => {
                self.emit(MirInstr::CallDirect(
                    dest,
                    "__bs_promise_static_resolve".to_string(),
                    mir_args,
                ));
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
