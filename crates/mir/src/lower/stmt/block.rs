use diagnostics::CompileResult;
use hir::HirStmt;
use super::super::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_block(&mut self, stmts: &[HirStmt]) -> CompileResult<()> {
        self.lower_stmts(stmts)
    }
}
