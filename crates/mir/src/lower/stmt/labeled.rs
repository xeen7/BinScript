use diagnostics::CompileResult;
use hir::HirStmt;
use crate::types::*;
use super::super::{LowerCtx, LoopStackFrame};

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_labeled(
        &mut self,
        label: &str,
        body: &HirStmt,
    ) -> CompileResult<()> {
        let old_label = self.next_loop_label.take();
        self.next_loop_label = Some(label.to_string());

        let is_loop = matches!(
            body,
            HirStmt::While { .. }
                | HirStmt::DoWhile { .. }
                | HirStmt::For { .. }
                | HirStmt::ForOf { .. }
        );

        if is_loop {
            self.lower_stmt(body)?;
        } else {
            let exit_bb = self.fresh_block();
            self.loop_stack.push(LoopStackFrame {
                label: Some(label.to_string()),
                continue_target: None,
                break_target: exit_bb,
            });
            self.lower_stmt(body)?;
            self.loop_stack.pop();
            if !self.current_block_terminated() {
                self.emit(MirInstr::Jump(exit_bb));
            }
            self.switch_to(exit_bb);
        }
        self.next_loop_label = old_label;
        Ok(())
    }
}
