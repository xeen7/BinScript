use diagnostics::CompileResult;
use hir::HirExpr;
use crate::types::*;
use super::super::LowerCtx;


impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_throw(&mut self, expr: &HirExpr) -> CompileResult<()> {
        let val = self.lower_expr(expr)?;
        self.emit(MirInstr::Throw(val));
        Ok(())
    }
}
