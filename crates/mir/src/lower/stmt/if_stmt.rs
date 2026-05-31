use diagnostics::CompileResult;
use hir::{HirStmt, HirExpr};
use crate::types::*;
use super::super::LowerCtx;


impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_if(
        &mut self,
        cond: &HirExpr,
        then_body: &[HirStmt],
        else_body: &Option<Vec<HirStmt>>,
    ) -> CompileResult<()> {
        let cv = self.lower_expr(cond)?;
        let then_bb = self.fresh_block();
        let else_bb = self.fresh_block();
        let merge_bb = self.fresh_block();

        self.emit(MirInstr::Branch(cv, then_bb, else_bb));

        self.switch_to(then_bb);
        self.lower_stmts(then_body)?;
        if !self.current_block_terminated() {
            self.emit(MirInstr::Jump(merge_bb));
        }

        self.switch_to(else_bb);
        if let Some(els) = else_body {
            self.lower_stmts(els)?;
        }
        if !self.current_block_terminated() {
            self.emit(MirInstr::Jump(merge_bb));
        }

        self.switch_to(merge_bb);
        Ok(())
    }
}
