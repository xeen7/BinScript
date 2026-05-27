use swc_core::ecma::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

mod decl;
mod expr_stmt;
mod if_stmt;
mod while_stmt;
pub mod for_stmt;
pub mod for_of;
pub mod for_in;
mod return_stmt;
mod block;
mod empty;
mod break_stmt;
mod continue_stmt;
mod do_while;
mod throw;
mod try_stmt;
mod switch_stmt;

impl LowerCtx {
    pub(crate) fn lower_stmt(&mut self, stmt: &Stmt, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        match stmt {
            Stmt::Decl(decl) => self.lower_stmt_decl(decl, out),
            Stmt::Expr(es) => self.lower_stmt_expr(es, out),
            Stmt::If(if_s) => self.lower_stmt_if(if_s, out),
            Stmt::While(w) => self.lower_stmt_while(w, out),
            Stmt::For(f) => self.lower_stmt_for(f, out),
            Stmt::ForOf(f) => self.lower_stmt_for_of(f, out),
            Stmt::ForIn(f) => self.lower_stmt_for_in(f, out),
            Stmt::Return(r) => self.lower_stmt_return(r, out),
            Stmt::Block(b) => self.lower_stmt_block(b, out),
            Stmt::Empty(e) => self.lower_stmt_empty(e, out),
            Stmt::Break(b) => self.lower_stmt_break(b, out),
            Stmt::Continue(c) => self.lower_stmt_continue(c, out),
            Stmt::DoWhile(dw) => self.lower_stmt_do_while(dw, out),
            Stmt::Throw(t) => self.lower_stmt_throw(t, out),
            Stmt::Try(t) => self.lower_stmt_try(t, out),
            Stmt::Switch(s) => self.lower_stmt_switch(s, out),
            Stmt::Debugger(_) => {
                // Treated as a compile-time no-op (breakpoint in a standard JS runtime)
                Ok(())
            }
            Stmt::Labeled(l) => {
                let label = l.label.sym.to_string();
                let mut body_stmts = Vec::new();
                self.lower_stmt(&l.body, &mut body_stmts)?;
                if !body_stmts.is_empty() {
                    if body_stmts.len() == 1 {
                        let wrapped = HirStmt::Labeled {
                            label,
                            body: Box::new(body_stmts.remove(0)),
                        };
                        out.push(wrapped);
                    } else {
                        let is_last_loop = matches!(
                            body_stmts.last().unwrap(),
                            HirStmt::While { .. }
                                | HirStmt::DoWhile { .. }
                                | HirStmt::For { .. }
                                | HirStmt::ForOf { .. }
                        );
                        if is_last_loop {
                            let last_stmt = body_stmts.pop().unwrap();
                            let wrapped = HirStmt::Labeled {
                                label,
                                body: Box::new(last_stmt),
                            };
                            out.extend(body_stmts);
                            out.push(wrapped);
                        } else {
                            let wrapped = HirStmt::Labeled {
                                label,
                                body: Box::new(HirStmt::Block(body_stmts)),
                            };
                            out.push(wrapped);
                        }
                    }
                }
                Ok(())
            }
            Stmt::With(w) => {
                self.lower_stmt(&w.body, out)
            }
        }
    }

    pub(crate) fn lower_stmt_to_vec(&mut self, stmt: &Stmt) -> CompileResult<Vec<HirStmt>> {
        match stmt {
            Stmt::Block(b) => {
                self.push_scope();
                let v = self.lower_block_stmts(b)?;
                self.pop_scope();
                Ok(v)
            }
            other => {
                let mut v = Vec::new();
                self.lower_stmt(other, &mut v)?;
                Ok(v)
            }
        }
    }

    pub(crate) fn lower_block_stmts(&mut self, block: &BlockStmt) -> CompileResult<Vec<HirStmt>> {
        let mut out = Vec::new();
        for s in &block.stmts {
            self.lower_stmt(s, &mut out)?;
        }
        Ok(out)
    }
}
