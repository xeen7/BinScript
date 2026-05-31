use diagnostics::CompileResult;
use hir::{HirStmt, HirExpr};
use crate::types::*;
use super::super::{LowerCtx, LoopStackFrame};


impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_for(
        &mut self,
        init: &Option<Box<HirStmt>>,
        cond: &Option<HirExpr>,
        update: &Option<HirExpr>,
        body: &[HirStmt],
    ) -> CompileResult<()> {
        if let Some(i) = init {
            self.lower_stmt(i)?;
        }

        let cond_bb = self.fresh_block();
        let body_bb = self.fresh_block();
        let update_bb = self.fresh_block();
        let exit_bb = self.fresh_block();

        self.emit(MirInstr::Jump(cond_bb));

        self.switch_to(cond_bb);
        if let Some(c) = cond {
            let cv = self.lower_expr(c)?;
            self.emit(MirInstr::Branch(cv, body_bb, exit_bb));
        } else {
            self.emit(MirInstr::Jump(body_bb));
        }

        let label = self.next_loop_label.take();
        self.loop_stack.push(LoopStackFrame {
            label,
            continue_target: Some(update_bb),
            break_target: exit_bb,
        });
        self.switch_to(body_bb);
        self.lower_stmts(body)?;
        if !self.current_block_terminated() {
            self.emit(MirInstr::Jump(update_bb));
        }
        self.loop_stack.pop();

        self.switch_to(update_bb);
        if let Some(u) = update {
            self.lower_expr(u)?;
        }
        self.emit(MirInstr::Jump(cond_bb));

        self.switch_to(exit_bb);
        Ok(())
    }
}
