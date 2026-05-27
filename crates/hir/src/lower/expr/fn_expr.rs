use swc_core::ecma::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_function(&mut self, function: &Function, name: String) -> CompileResult<HirExpr> {
        let func_id = self.fresh_func_id();

        self.function_stack.push(func_id);
        self.push_scope();
        let mut params = Vec::new();
        let mut param_destruct_stmts = Vec::new();
        for (param_idx, p) in function.params.iter().enumerate() {
            match &p.pat {
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
        
        let body = match &function.body {
            Some(block) => self.lower_block_stmts(block)?,
            None => Vec::new(),
        };
        let mut full_body = param_destruct_stmts;
        full_body.extend(body);
        
        self.pop_scope();
        self.function_stack.pop();

        let unique_name = format!("{}_{}", name, func_id);

        self.functions.push(HirFunction {
            id: func_id,
            name: unique_name,
            params: params.clone(),
            body: full_body,
            captures: Vec::new(),
            is_generator: function.is_generator,
            is_async: function.is_async,
        });

        Ok(HirExpr::Closure {
            func_id,
            captures: Vec::new(),
        })
    }

    pub(super) fn lower_expr_fn(&mut self, fn_expr: &FnExpr) -> CompileResult<HirExpr> {
        let name = fn_expr.ident.as_ref()
            .map(|id| id.sym.to_string())
            .unwrap_or_else(|| format!("__bs_closure_{}", self.fresh_func_id()));
        self.lower_function(&fn_expr.function, name)
    }
}
