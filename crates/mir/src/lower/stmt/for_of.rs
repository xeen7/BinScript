use diagnostics::CompileResult;
use hir::{HirStmt, HirExpr};
use crate::types::*;
use super::super::{LowerCtx, LoopStackFrame};
use crate::lower::builtins::BuiltinFn;


impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_for_of(
        &mut self,
        left: &HirStmt,
        right: &HirExpr,
        body: &[HirStmt],
        is_await: bool,
    ) -> CompileResult<()> {
        let iter_reg = self.fresh_reg();
        let iter_val = self.lower_expr(right)?;
        self.emit(MirInstr::Move(iter_reg, iter_val));

        let cond_bb = self.fresh_block();
        let body_bb = self.fresh_block();
        let exit_bb = self.fresh_block();

        self.emit(MirInstr::Jump(cond_bb));
        self.switch_to(cond_bb);

        // Call generator.next()
        let next_val_reg = self.fresh_reg();
        self.emit(MirInstr::CallBuiltin(
            next_val_reg,
            BuiltinFn::GeneratorNext,
            vec![MirOperand::Reg(iter_reg), MirOperand::ConstUndefined],
        ));

        let resolved_val_reg = if is_await {
            let yield_idx = self.num_yield_points;
            self.num_yield_points += 1;

            let mut saves = Vec::new();
            for r in 0..self.next_reg {
                saves.push(r);
            }
            self.yield_saves.push(saves);

            self.emit(MirInstr::Suspend(yield_idx, MirOperand::Reg(next_val_reg)));
            let dest = self.fresh_reg();
            self.emit(MirInstr::Resume(dest, yield_idx));
            dest
        } else {
            next_val_reg
        };

        // Check if iterator is done
        let is_done_reg = self.fresh_reg();
        self.emit(MirInstr::CallBuiltin(
            is_done_reg,
            BuiltinFn::GeneratorIsDone,
            vec![MirOperand::Reg(iter_reg)],
        ));
        self.emit(MirInstr::Branch(MirOperand::Reg(is_done_reg), exit_bb, body_bb));

        self.switch_to(body_bb);
        let label = self.next_loop_label.take();
        self.loop_stack.push(LoopStackFrame {
            label,
            continue_target: Some(cond_bb),
            break_target: exit_bb,
        });

        // Declare the loop variable if it's a Let, or evaluate the assignee
        self.lower_stmt(left)?;

        // Assign to left
        match left {
            HirStmt::Expr(HirExpr::Assign { target, .. }) => {
                let reg = self.bindings[target];
                if self.capture_cells.contains(target) {
                    self.emit(MirInstr::StoreField(reg, 0, MirOperand::Reg(resolved_val_reg)));
                } else {
                    self.emit(MirInstr::Move(reg, MirOperand::Reg(resolved_val_reg)));
                }
            }
            HirStmt::Let { binding, name: _, init: _ } => {
                let reg = self.bindings[binding];
                if self.capture_cells.contains(binding) {
                    self.emit(MirInstr::StoreField(reg, 0, MirOperand::Reg(resolved_val_reg)));
                } else {
                    self.emit(MirInstr::Move(reg, MirOperand::Reg(resolved_val_reg)));
                }
            }
            _ => unreachable!("for..of left is neither assignment nor let"),
        }

        self.lower_stmts(body)?;

        if !self.current_block_terminated() {
            self.emit(MirInstr::Jump(cond_bb));
        }

        self.loop_stack.pop();
        self.switch_to(exit_bb);

        Ok(())
    }
}
