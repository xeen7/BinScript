use diagnostics::CompileResult;
use hir::HirExpr;
use crate::types::*;
use super::super::{LowerCtx, LoopStackFrame};


impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_switch(
        &mut self,
        discriminant: &HirExpr,
        cases: &[hir::HirSwitchCase],
    ) -> CompileResult<()> {
        let disc_operand = self.lower_expr(discriminant)?;
        let exit_bb = self.fresh_block();

        let mut body_bbs = Vec::new();
        for _ in cases {
            body_bbs.push(self.fresh_block());
        }

        self.loop_stack.push(LoopStackFrame {
            label: None,
            continue_target: None,
            break_target: exit_bb,
        });

        let mut current_test_bb = self.current;
        let mut default_idx = None;

        for (i, case) in cases.iter().enumerate() {
            if let Some(test_expr) = &case.test {
                let next_test_bb = self.fresh_block();
                self.switch_to(current_test_bb);

                let test_operand = self.lower_expr(test_expr)?;
                let eq_reg = self.fresh_reg();
                self.emit(MirInstr::StrictEq(eq_reg, disc_operand.clone(), test_operand));
                self.emit(MirInstr::Branch(MirOperand::Reg(eq_reg), body_bbs[i], next_test_bb));

                current_test_bb = next_test_bb;
            } else {
                default_idx = Some(i);
            }
        }

        self.switch_to(current_test_bb);
        if let Some(def_i) = default_idx {
            self.emit(MirInstr::Jump(body_bbs[def_i]));
        } else {
            self.emit(MirInstr::Jump(exit_bb));
        }

        for (i, case) in cases.iter().enumerate() {
            self.switch_to(body_bbs[i]);
            self.lower_stmts(&case.consequent)?;

            if !self.current_block_terminated() {
                let next_bb = if i + 1 < cases.len() {
                    body_bbs[i + 1]
                } else {
                    exit_bb
                };
                self.emit(MirInstr::Jump(next_bb));
            }
        }

        self.loop_stack.pop();
        self.switch_to(exit_bb);
        Ok(())
    }
}
