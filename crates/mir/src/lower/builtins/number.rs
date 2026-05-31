use diagnostics::CompileResult;
use crate::types::*;
use super::super::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_builtin_number(
        &mut self,
        method: &str,
        mir_args: Vec<MirOperand>,
        dest: MirReg,
    ) -> CompileResult<bool> {
        match method {
            "isInteger" => {
                self.emit(MirInstr::CallDirect(dest, "__bs_number_isInteger".to_string(), mir_args));
                Ok(true)
            }
            "isFinite" => {
                self.emit(MirInstr::CallDirect(dest, "__bs_isFinite".to_string(), mir_args));
                Ok(true)
            }
            "isNaN" => {
                self.emit(MirInstr::CallDirect(dest, "__bs_isNaN".to_string(), mir_args));
                Ok(true)
            }
            "isSafeInteger" => {
                self.emit(MirInstr::CallDirect(dest, "__bs_number_isSafeInteger".to_string(), mir_args));
                Ok(true)
            }
            "parseInt" => {
                let mut padded_args = mir_args;
                while padded_args.len() < 2 {
                    padded_args.push(MirOperand::ConstUndefined);
                }
                self.emit(MirInstr::CallDirect(dest, "__bs_parseInt".to_string(), padded_args));
                Ok(true)
            }
            "parseFloat" => {
                let mut padded_args = mir_args;
                if padded_args.is_empty() {
                    padded_args.push(MirOperand::ConstUndefined);
                }
                self.emit(MirInstr::CallDirect(dest, "__bs_parseFloat".to_string(), padded_args));
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
