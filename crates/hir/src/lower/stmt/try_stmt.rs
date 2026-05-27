use swc_core::ecma::ast::*;

use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_stmt_try(&mut self, t: &TryStmt, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        let mut catch_param = None;
        if let Some(handler) = &t.handler {
            if let Some(param) = &handler.param {
                if let Pat::Ident(binding_ident) = param {
                    let name = binding_ident.id.sym.to_string();
                    let bid = self.next_binding;
                    self.next_binding += 1;
                    
                    let current_func = *self.function_stack.last().unwrap_or(&0);
                    self.binding_owners.insert(bid, current_func);
                    
                    catch_param = Some((bid, name));
                } else {
                    return Err(CompileError::Lowering {
                        message: "Unsupported complex pattern in catch parameter".into(),
                    });
                }
            }
        }

        self.push_scope();
        let body = self.lower_block_stmts(&t.block)?;
        self.pop_scope();

        let catch_body = if let Some(handler) = &t.handler {
            self.push_scope();
            if let Some((bid, ref name)) = catch_param {
                self.insert_binding(name.clone(), bid);
            }
            let stmts = self.lower_block_stmts(&handler.body)?;
            self.pop_scope();
            stmts
        } else {
            Vec::new()
        };

        let finally_body = if let Some(finalizer) = &t.finalizer {
            self.push_scope();
            let stmts = self.lower_block_stmts(finalizer)?;
            self.pop_scope();
            Some(stmts)
        } else {
            None
        };

        out.push(HirStmt::Try {
            body,
            catch_param,
            catch_body,
            finally_body,
        });

        Ok(())
    }
}
