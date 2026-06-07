use oxc::ast::ast::*;
use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_class(&mut self, ce: &Class) -> CompileResult<HirExpr> {
        let name = ce.id.as_ref()
            .map(|id| id.name.to_string())
            .unwrap_or_else(|| format!("_AnonClass_{}", self.fresh_func_id()));
        self.lower_expr_class_with_name(ce, name)
    }

    pub(crate) fn lower_expr_class_with_name(&mut self, ce: &Class, class_name: String) -> CompileResult<HirExpr> {
        let func_id = self.fresh_func_id();
        self.function_stack.push(func_id);
        self.push_scope();

        let mut body = Vec::new();
        
        self.lower_class_decl(ce, Some(class_name.clone()), &mut body)?;

        let binding = self.lookup(&class_name).unwrap();

        body.push(HirStmt::Return(Some(HirExpr::Var(binding))));

        self.pop_scope();
        self.function_stack.pop();

        self.functions.push(HirFunction {
            id: func_id,
            name: format!("__bs_class_expr_{}", func_id),
            params: vec![],
            body,
            captures: Vec::new(),
            is_generator: false,
            is_async: false,
        });

        let closure = HirExpr::Closure { func_id, captures: Vec::new() };
        Ok(HirExpr::Call {
            callee: Box::new(closure),
            args: vec![],
        })
    }
}
