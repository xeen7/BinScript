use diagnostics::CompileResult;
use hir::{HirStmt, BindingId};
use crate::types::*;
use super::super::LowerCtx;


impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_try(
        &mut self,
        body: &[HirStmt],
        catch_param: &Option<(BindingId, String)>,
        catch_body: &[HirStmt],
        finally_body: &Option<Vec<HirStmt>>,
    ) -> CompileResult<()> {
        if let Some(fin_stmts) = finally_body {
            let finally_caught_reg = self.fresh_reg();
            let finally_err_reg = self.fresh_reg();
            self.emit(MirInstr::Move(finally_caught_reg, MirOperand::ConstBool(false)));

            let jmp_buf_reg = self.fresh_reg();
            self.emit(MirInstr::TryEnter(jmp_buf_reg));

            let setjmp_result = self.fresh_reg();
            self.emit(MirInstr::SetJmp(setjmp_result, jmp_buf_reg));

            let try_body_bb = self.fresh_block();
            let catch_bb = self.fresh_block();
            let finally_bb = self.fresh_block();

            self.emit(MirInstr::Branch(MirOperand::Reg(setjmp_result), catch_bb, try_body_bb));

            self.switch_to(try_body_bb);

            let has_catch = catch_param.is_some() || !catch_body.is_empty();
            if has_catch {
                self.lower_stmt_try(body, catch_param, catch_body, &None)?;
            } else {
                self.lower_stmts(body)?;
            }

            if !self.current_block_terminated() {
                self.emit(MirInstr::TryExit);
                self.emit(MirInstr::Jump(finally_bb));
            }

            self.switch_to(catch_bb);
            self.emit(MirInstr::CallDirect(
                finally_err_reg,
                "__bs_get_and_clear_exception".to_string(),
                vec![],
            ));
            self.emit(MirInstr::Move(finally_caught_reg, MirOperand::ConstBool(true)));
            self.emit(MirInstr::Jump(finally_bb));

            self.switch_to(finally_bb);
            self.lower_stmts(fin_stmts)?;

            let rethrow_bb = self.fresh_block();
            let merge_bb = self.fresh_block();
            self.emit(MirInstr::Branch(MirOperand::Reg(finally_caught_reg), rethrow_bb, merge_bb));

            self.switch_to(rethrow_bb);
            self.emit(MirInstr::Throw(MirOperand::Reg(finally_err_reg)));

            self.switch_to(merge_bb);
        } else {
            let jmp_buf_reg = self.fresh_reg();
            self.emit(MirInstr::TryEnter(jmp_buf_reg));

            let setjmp_result = self.fresh_reg();
            self.emit(MirInstr::SetJmp(setjmp_result, jmp_buf_reg));

            let try_body_bb = self.fresh_block();
            let catch_bb = self.fresh_block();
            let merge_bb = self.fresh_block();

            self.emit(MirInstr::Branch(MirOperand::Reg(setjmp_result), catch_bb, try_body_bb));

            self.switch_to(try_body_bb);
            self.lower_stmts(body)?;
            if !self.current_block_terminated() {
                self.emit(MirInstr::TryExit);
                self.emit(MirInstr::Jump(merge_bb));
            }

            self.switch_to(catch_bb);
            if let Some((bid, _name)) = catch_param {
                let exc_reg = self.fresh_reg();
                self.bind(*bid, exc_reg);
                self.emit(MirInstr::CallDirect(
                    exc_reg,
                    "__bs_get_and_clear_exception".to_string(),
                    vec![],
                ));
            } else {
                let unused = self.fresh_reg();
                self.emit(MirInstr::CallDirect(
                    unused,
                    "__bs_get_and_clear_exception".to_string(),
                    vec![],
                ));
            }
            self.lower_stmts(catch_body)?;
            if !self.current_block_terminated() {
                self.emit(MirInstr::Jump(merge_bb));
            }

            self.switch_to(merge_bb);
        }
        Ok(())
    }
}
