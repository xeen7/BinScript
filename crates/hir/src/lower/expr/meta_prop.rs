use oxc::ast::ast::*;
use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_meta_prop(&mut self, mp: &MetaProperty) -> CompileResult<HirExpr> {
        let meta_name = mp.meta.name.as_str();
        let prop_name = mp.property.name.as_str();
        
        if meta_name == "new" && prop_name == "target" {
            Ok(HirExpr::Lit(Literal::Undefined))
        } else if meta_name == "import" && prop_name == "meta" {
            let func_id = self.fresh_func_id();
            self.function_stack.push(func_id);
            self.push_scope();

            let mut body = Vec::new();
            let obj_name = format!("_meta_obj_{}", func_id);
            let obj_binding = self.declare(&obj_name);
            body.push(HirStmt::Let {
                binding: obj_binding,
                name: obj_name,
                init: Some(HirExpr::Call {
                    callee: Box::new(HirExpr::GlobalRef("__bs_new_object".to_string())),
                    args: vec![],
                }),
            });

            body.push(HirStmt::Expr(HirExpr::MemberSet {
                object: Box::new(HirExpr::Var(obj_binding)),
                property: "url".to_string(),
                value: Box::new(HirExpr::Lit(Literal::String("file:///main.ts".to_string()))),
            }));

            body.push(HirStmt::Return(Some(HirExpr::Var(obj_binding))));

            self.pop_scope();
            self.function_stack.pop();

            self.functions.push(HirFunction {
                id: func_id,
                name: format!("__bs_import_meta_{}", func_id),
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
        } else {
            Ok(HirExpr::Lit(Literal::Undefined))
        }
    }
}
