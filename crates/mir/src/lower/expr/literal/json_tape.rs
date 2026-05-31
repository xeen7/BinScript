use diagnostics::CompileResult;
use crate::lower::builtins::BuiltinFn;
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_json_tape(&mut self, bytes: &[u8]) -> CompileResult<MirOperand> {
        let s = String::from_utf8_lossy(bytes).into_owned();
        let dest = self.fresh_reg();
        // Emitting it as a builtin/intrinsic call. JsonParseLazy is handled in codegen.
        self.emit(MirInstr::CallBuiltin(dest, BuiltinFn::JsonParseLazy, vec![MirOperand::ConstStr(s)]));
        Ok(MirOperand::Reg(dest))
    }
}
