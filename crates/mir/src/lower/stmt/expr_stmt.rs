use diagnostics::CompileResult;
use hir::HirExpr;
use super::super::LowerCtx;


impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_expr(&mut self, e: &HirExpr) -> CompileResult<()> {
        self.lower_expr(e)?;
        Ok(())
    }
}
