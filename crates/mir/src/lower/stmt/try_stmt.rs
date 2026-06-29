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
        let catch_bb = self.fresh_block();
        let finally_bb = if finally_body.is_some() { Some(self.fresh_block()) } else { None };
        let merge_bb = self.fresh_block();
        
        // Scope ID for TryEnter
        let scope_id = self.next_scope_id();
        
        // If there's a finally block, we need to know if we arrived there via catch/normal vs an uncaught exception
        let finally_caught_reg = self.fresh_reg();
        let finally_err_reg = self.fresh_reg();
        
        if finally_body.is_some() {
            self.emit(MirInstr::Move(finally_caught_reg, MirOperand::ConstBool(false)));
        }

        self.emit(MirInstr::TryEnter {
            scope_id,
            catch_target: catch_bb,
        });

        // The body falls through
        self.active_exception_scopes.push((scope_id, catch_bb));
        self.lower_stmts(body)?;
        self.active_exception_scopes.pop();

        // Always emit TryExit at the end of the try body to manage runtime scope stack
        self.emit(MirInstr::TryExit);

        if !self.current_block_terminated() {
            self.emit(MirInstr::Jump(finally_bb.unwrap_or(merge_bb)));
        }

        // --- Catch block ---
        self.switch_to(catch_bb);
        let lp_reg = self.fresh_reg();
        self.emit(MirInstr::LandingPad { exn_reg: lp_reg, is_cleanup: false });
        
        if let Some((bid, _name)) = catch_param {
            let exc_reg = self.fresh_reg();
            self.bind(*bid, exc_reg);
            self.emit(MirInstr::ExtractException { dest: exc_reg, lp_reg });
        } else {
            let unused = self.fresh_reg();
            self.emit(MirInstr::ExtractException { dest: unused, lp_reg });
        }

        // If there's a finally block, any throw from the catch body must route
        // through a cleanup landing pad that saves the exception and jumps to finally.
        if let Some(fin_bb) = finally_bb {
            let catch_cleanup_bb = self.fresh_block();
            let catch_cleanup_scope = self.next_scope_id();
            self.emit(MirInstr::TryEnter {
                scope_id: catch_cleanup_scope,
                catch_target: catch_cleanup_bb,
            });
            self.active_exception_scopes.push((catch_cleanup_scope, catch_cleanup_bb));
            self.lower_stmts(catch_body)?;
            self.active_exception_scopes.pop();
            self.emit(MirInstr::TryExit);

            if !self.current_block_terminated() {
                self.emit(MirInstr::Jump(fin_bb));
            }

            // --- Cleanup landing pad for exceptions thrown from catch body ---
            self.switch_to(catch_cleanup_bb);
            let cleanup_lp_reg = self.fresh_reg();
            self.emit(MirInstr::LandingPad { exn_reg: cleanup_lp_reg, is_cleanup: false });
            let cleanup_exc_reg = self.fresh_reg();
            self.emit(MirInstr::ExtractException { dest: cleanup_exc_reg, lp_reg: cleanup_lp_reg });
            // Save the exception for rethrow after finally
            self.emit(MirInstr::Move(finally_err_reg, MirOperand::Reg(cleanup_exc_reg)));
            self.emit(MirInstr::Move(finally_caught_reg, MirOperand::ConstBool(true)));
            self.emit(MirInstr::Jump(fin_bb));
        } else {
            self.lower_stmts(catch_body)?;

            if !self.current_block_terminated() {
                self.emit(MirInstr::Jump(merge_bb));
            }
        }

        // --- Finally block ---
        if let Some(fin_bb) = finally_bb {
            self.switch_to(fin_bb);
            self.lower_stmts(finally_body.as_ref().unwrap())?;

            let rethrow_bb = self.fresh_block();
            self.emit(MirInstr::Branch(MirOperand::Reg(finally_caught_reg), rethrow_bb, merge_bb));

            self.switch_to(rethrow_bb);
            self.emit(MirInstr::Throw(MirOperand::Reg(finally_err_reg)));
        }

        self.switch_to(merge_bb);
        Ok(())
    }
}
