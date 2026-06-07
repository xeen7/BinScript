use oxc::ast::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_arrow(&mut self, arrow: &ArrowFunctionExpression) -> CompileResult<HirExpr> {
        let func_id = self.fresh_func_id();
        let name = format!("__bs_closure_{}", func_id);

        self.function_stack.push(func_id);
        self.push_scope();
        
        let mut params = Vec::new();
        let mut param_destruct_stmts = Vec::new();
        self.lower_formal_parameters(&arrow.params, &mut params, &mut param_destruct_stmts)?;
        
        let body = if arrow.expression && arrow.body.statements.len() == 1 {
            if let Statement::ExpressionStatement(expr_stmt) = &arrow.body.statements[0] {
                let val = self.lower_expr(&expr_stmt.expression)?;
                vec![HirStmt::Return(Some(val))]
            } else {
                self.lower_function_body(&arrow.body)?
            }
        } else {
            self.lower_function_body(&arrow.body)?
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
            is_generator: false,
            is_async: arrow.r#async,
        });

        Ok(HirExpr::Closure {
            func_id,
            captures: Vec::new(),
        })
    }
}
