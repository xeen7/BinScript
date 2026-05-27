use swc_core::ecma::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_arrow(&mut self, arrow: &ArrowExpr) -> CompileResult<HirExpr> {
        let func_id = self.fresh_func_id();
        let name = format!("__bs_closure_{}", func_id);

        self.function_stack.push(func_id);
        self.push_scope();
        
        let mut params = Vec::new();
        let mut param_destruct_stmts = Vec::new();
        for (param_idx, p) in arrow.params.iter().enumerate() {
            match p {
                Pat::Ident(ident) => {
                    let pname = ident.sym.to_string();
                    let pid = self.declare(&pname);
                    params.push((pid, pname));
                }
                other_pat => {
                    let pname = format!("_param_{}", param_idx);
                    let pid = self.declare(&pname);
                    params.push((pid, pname.clone()));
                    self.lower_pattern(other_pat, HirExpr::Var(pid), &mut param_destruct_stmts)?;
                }
            }
        }
        
        let body = match &*arrow.body {
            BlockStmtOrExpr::BlockStmt(block) => self.lower_block_stmts(block)?,
            BlockStmtOrExpr::Expr(expr) => {
                let val = self.lower_expr(expr)?;
                vec![HirStmt::Return(Some(val))]
            }
        };
        let mut full_body = param_destruct_stmts;
        full_body.extend(body);
        
        self.pop_scope();
        self.function_stack.pop();

        self.functions.push(HirFunction {
            id: func_id,
            name,
            params,
            body: full_body,
            captures: Vec::new(),
            is_generator: arrow.is_generator,
            is_async: arrow.is_async,
        });

        Ok(HirExpr::Closure {
            func_id,
            captures: Vec::new(),
        })
    }
}
