use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_builtin_json(
        &mut self,
        method: &str,
        mir_args: Vec<MirOperand>,
        dest: MirReg,
    ) -> CompileResult<bool> {
        match method {
            "stringify" => {
                let arg = mir_args.into_iter().next().unwrap_or(MirOperand::ConstUndefined);
                self.emit(MirInstr::CallDirect(
                    dest,
                    "__bs_json_stringify".to_string(),
                    vec![arg],
                ));
                Ok(true)
            }
            "parse" => {
                let arg = mir_args.into_iter().next().unwrap_or(MirOperand::ConstUndefined);
                self.emit(MirInstr::CallDirect(
                    dest,
                    "__bs_json_parse".to_string(),
                    vec![arg],
                ));
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
