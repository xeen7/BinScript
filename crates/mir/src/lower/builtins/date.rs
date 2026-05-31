use diagnostics::CompileResult;
use crate::types::*;
use super::super::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_builtin_date(
        &mut self,
        method: &str,
        mir_args: Vec<MirOperand>,
        dest: MirReg,
    ) -> CompileResult<bool> {
        match method {
            "now" => {
                self.emit(MirInstr::CallDirect(dest, "__bs_date_now".to_string(), mir_args));
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
