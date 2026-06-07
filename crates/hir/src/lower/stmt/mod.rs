use oxc::ast::ast::*;

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
    pub(crate) fn lower_stmt(&mut self, stmt: &Statement, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        // In OXC, Declaration variants are flattened into Statement via inherit_variants!
        if let Some(decl) = stmt.as_declaration() {
            return self.lower_stmt_decl(decl, out);
        }
        match stmt {
            Statement::ExpressionStatement(es) => self.lower_stmt_expr(es, out),
            Statement::IfStatement(if_s) => self.lower_stmt_if(if_s, out),
            Statement::WhileStatement(w) => self.lower_stmt_while(w, out),
            Statement::ForStatement(f) => self.lower_stmt_for(f, out),
            Statement::ForOfStatement(f) => self.lower_stmt_for_of(f, out),
            Statement::ForInStatement(f) => self.lower_stmt_for_in(f, out),
            Statement::ReturnStatement(r) => self.lower_stmt_return(r, out),
            Statement::BlockStatement(b) => self.lower_stmt_block(b, out),
            Statement::EmptyStatement(e) => self.lower_stmt_empty(e, out),
            Statement::BreakStatement(b) => self.lower_stmt_break(b, out),
            Statement::ContinueStatement(c) => self.lower_stmt_continue(c, out),
            Statement::DoWhileStatement(dw) => self.lower_stmt_do_while(dw, out),
            Statement::ThrowStatement(t) => self.lower_stmt_throw(t, out),
            Statement::TryStatement(t) => self.lower_stmt_try(t, out),
            Statement::SwitchStatement(s) => self.lower_stmt_switch(s, out),
            Statement::DebuggerStatement(_) => {
                // Treated as a compile-time no-op (breakpoint in a standard JS runtime)
                Ok(())
            }
            Statement::LabeledStatement(l) => {
                let label = l.label.name.to_string();
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
            Statement::WithStatement(w) => {
                self.lower_stmt(&w.body, out)
            }
            // Module declarations (import/export) are handled by lower_module_decl
            _ => Ok(()),
        }
    }

    pub(crate) fn lower_stmt_to_vec(&mut self, stmt: &Statement) -> CompileResult<Vec<HirStmt>> {
        match stmt {
            Statement::BlockStatement(b) => {
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

    pub(crate) fn lower_block_stmts(&mut self, block: &BlockStatement) -> CompileResult<Vec<HirStmt>> {
        let mut out = Vec::new();
        for s in &block.body {
            self.lower_stmt(s, &mut out)?;
        }
        Ok(out)
    }

    pub(crate) fn lower_function_body(&mut self, body: &FunctionBody) -> CompileResult<Vec<HirStmt>> {
        let mut out = Vec::new();
        for s in &body.statements {
            self.lower_stmt(s, &mut out)?;
        }
        Ok(out)
    }
}
