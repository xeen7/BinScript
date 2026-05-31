use diagnostics::CompileResult;
use hir::HirStmt;
use super::LowerCtx;

mod expr_stmt;
mod let_stmt;
mod assign_stmt;
mod if_stmt;
mod while_stmt;
mod do_while;
mod for_stmt;
mod for_of;
mod return_stmt;
mod break_stmt;
mod continue_stmt;
mod switch;
mod block;
mod labeled;
mod func_decl;
mod throw;
mod try_stmt;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmts(&mut self, stmts: &[HirStmt]) -> CompileResult<()> {
        for s in stmts {
            self.lower_stmt(s)?;
            // Stop emitting after a terminator (throw, return, break, continue)
            if self.current_block_terminated() {
                break;
            }
        }
        Ok(())
    }

    pub(super) fn lower_stmt(&mut self, stmt: &HirStmt) -> CompileResult<()> {
        match stmt {
            HirStmt::Expr(e)                                                     => self.lower_stmt_expr(e),
            HirStmt::Let { binding, name, init }                                 => self.lower_stmt_let(binding, name, init),
            HirStmt::Assign { target, value }                                    => self.lower_stmt_assign(target, value),
            HirStmt::If { cond, then_body, else_body }                           => self.lower_stmt_if(cond, then_body, else_body),
            HirStmt::While { cond, body }                                        => self.lower_stmt_while(cond, body),
            HirStmt::DoWhile { body, cond }                                      => self.lower_stmt_do_while(body, cond),
            HirStmt::For { init, cond, update, body }                            => self.lower_stmt_for(init, cond, update, body),
            HirStmt::ForOf { left, right, body, is_await }                       => self.lower_stmt_for_of(left, right, body, *is_await),
            HirStmt::Return(val)                                                  => self.lower_stmt_return(val),
            HirStmt::Break(label)                                                 => self.lower_stmt_break(label),
            HirStmt::Continue(label)                                              => self.lower_stmt_continue(label),
            HirStmt::Switch { discriminant, cases }                              => self.lower_stmt_switch(discriminant, cases),
            HirStmt::Block(stmts)                                                => self.lower_stmt_block(stmts),
            HirStmt::Labeled { label, body }                                     => self.lower_stmt_labeled(label, body),
            HirStmt::FuncDecl { .. }                                             => self.lower_stmt_func_decl(),
            HirStmt::Throw(expr)                                                  => self.lower_stmt_throw(expr),
            HirStmt::Try { body, catch_param, catch_body, finally_body }         => self.lower_stmt_try(body, catch_param, catch_body, finally_body),
        }
    }
}
