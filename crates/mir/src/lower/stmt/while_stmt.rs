use diagnostics::CompileResult;
use hir::{HirStmt, HirExpr};
use crate::types::*;
use super::super::{LowerCtx, LoopStackFrame};


impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_while(
        &mut self,
        cond: &HirExpr,
        body: &[HirStmt],
    ) -> CompileResult<()> {
        let cond_bb = self.fresh_block();
        let body_bb = self.fresh_block();
        let exit_bb = self.fresh_block();

        self.emit(MirInstr::Jump(cond_bb));

        self.switch_to(cond_bb);
        let cv = self.lower_expr(cond)?;
        self.emit(MirInstr::Branch(cv, body_bb, exit_bb));

        let label = self.next_loop_label.take();
        self.loop_stack.push(LoopStackFrame {
            label,
            continue_target: Some(cond_bb),
            break_target: exit_bb,
        });
        self.switch_to(body_bb);
        self.lower_stmts(body)?;
        if !self.current_block_terminated() {
            self.emit(MirInstr::Jump(cond_bb));
        }
        self.loop_stack.pop();

        self.switch_to(exit_bb);
        Ok(())
    }
}
