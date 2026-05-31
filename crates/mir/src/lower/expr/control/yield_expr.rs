use diagnostics::CompileResult;
use hir::HirExpr;
use crate::lower::builtins::BuiltinFn;
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_yield(
        &mut self,
        arg: &Option<Box<HirExpr>>,
        delegate: bool,
    ) -> CompileResult<MirOperand> {
        if delegate {
            let iter_val = match arg {
                Some(e) => self.lower_expr(e)?,
                None => MirOperand::ConstUndefined,
            };
            let iter_reg = self.fresh_reg();
            self.emit(MirInstr::Move(iter_reg, iter_val));

            let sent_reg = self.fresh_reg();
            self.emit(MirInstr::Move(sent_reg, MirOperand::ConstUndefined));

            let result_reg = self.fresh_reg();

            let cond_bb = self.fresh_block();
            let body_bb = self.fresh_block();
            let exit_bb = self.fresh_block();

            self.emit(MirInstr::Jump(cond_bb));
            self.switch_to(cond_bb);

            let next_val_reg = self.fresh_reg();
            self.emit(MirInstr::CallBuiltin(
                next_val_reg,
                BuiltinFn::GeneratorNext,
                vec![MirOperand::Reg(iter_reg), MirOperand::Reg(sent_reg)],
            ));

            let is_done_reg = self.fresh_reg();
            self.emit(MirInstr::CallBuiltin(
                is_done_reg,
                BuiltinFn::GeneratorIsDone,
                vec![MirOperand::Reg(iter_reg)],
            ));
            self.emit(MirInstr::Branch(MirOperand::Reg(is_done_reg), exit_bb, body_bb));

            // Body
            self.switch_to(body_bb);

            let yield_idx = self.num_yield_points;
            self.num_yield_points += 1;
            let mut saves = Vec::new();
            for r in 0..self.next_reg {
                saves.push(r);
            }
            self.yield_saves.push(saves);

            self.emit(MirInstr::Suspend(yield_idx, MirOperand::Reg(next_val_reg)));
            self.emit(MirInstr::Resume(sent_reg, yield_idx));
            self.emit(MirInstr::Jump(cond_bb));

            // Exit
            self.switch_to(exit_bb);
            self.emit(MirInstr::Move(result_reg, MirOperand::Reg(next_val_reg)));

            Ok(MirOperand::Reg(result_reg))
        } else {
            let v = match arg {
                Some(e) => self.lower_expr(e)?,
                None => MirOperand::ConstUndefined,
            };
            let yield_idx = self.num_yield_points;
            self.num_yield_points += 1;

            // Save all registers assigned up to this point
            let mut saves = Vec::new();
            for r in 0..self.next_reg {
                saves.push(r);
            }
            self.yield_saves.push(saves);

            self.emit(MirInstr::Suspend(yield_idx, v));
            let dest = self.fresh_reg();
            self.emit(MirInstr::Resume(dest, yield_idx));
            Ok(MirOperand::Reg(dest))
        }
    }
}
