use diagnostics::CompileResult;
use super::super::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_func_decl(&self) -> CompileResult<()> {
        Ok(()) // handled at module level
    }
}
