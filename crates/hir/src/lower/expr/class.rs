use swc_core::ecma::ast::*;
use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_class(&mut self, ce: &ClassExpr) -> CompileResult<HirExpr> {
        let name = ce.ident.as_ref()
            .map(|id| id.sym.to_string())
            .unwrap_or_else(|| format!("_AnonClass_{}", self.fresh_func_id()));
        self.lower_expr_class_with_name(ce, name)
    }

    pub(crate) fn lower_expr_class_with_name(&mut self, ce: &ClassExpr, class_name: String) -> CompileResult<HirExpr> {
        let func_id = self.fresh_func_id();
        self.function_stack.push(func_id);
        self.push_scope();

        let mut body = Vec::new();
        
        let dummy_ident = Ident::new(
            class_name.clone().into(),
            swc_core::common::DUMMY_SP,
            swc_core::common::SyntaxContext::empty(),
        );
        let class_decl = ClassDecl {
            ident: dummy_ident,
            declare: false,
            class: ce.class.clone(),
        };

        self.lower_decl(&Decl::Class(class_decl), &mut body)?;

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
