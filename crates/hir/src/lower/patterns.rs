//! Pattern (destructuring) lowering.

use oxc::ast::ast::*;
use diagnostics::CompileResult;
use crate::types::*;
use super::context::LowerCtx;

impl LowerCtx {
    pub(crate) fn prop_name_to_string(&self, prop: &PropertyKey) -> String {
        match prop {
            PropertyKey::StaticIdentifier(id) => id.name.to_string(),
            PropertyKey::StringLiteral(s) => s.value.to_string(),
            PropertyKey::NumericLiteral(n) => n.value.to_string(),
            PropertyKey::PrivateIdentifier(id) => format!("__private_{}", id.name),
            _ => "unknown".to_string(),
        }
    }

    pub(crate) fn lower_pattern(&mut self, pat: &BindingPattern, base_expr: HirExpr, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        match pat {
            BindingPattern::BindingIdentifier(binding_ident) => {
                let name = binding_ident.name.to_string();
                let binding = self.declare(&name);
                out.push(HirStmt::Let {
                    binding,
                    name,
                    init: Some(base_expr),
                });
                Ok(())
            }
            BindingPattern::ArrayPattern(array_pat) => {
                for (idx, elem_opt) in array_pat.elements.iter().enumerate() {
                    if let Some(elem_pat) = elem_opt {
                        let elem_expr = HirExpr::IndexGet {
                            object: Box::new(base_expr.clone()),
                            index: Box::new(HirExpr::Lit(Literal::Number(idx as f64))),
                        };
                        self.lower_pattern(elem_pat, elem_expr, out)?;
                    }
                }
                // Handle rest element
                if let Some(rest) = &array_pat.rest {
                    let rest_expr = HirExpr::MethodCall {
                        object: Box::new(base_expr.clone()),
                        method: "slice".to_string(),
                        args: vec![
                            HirExpr::Lit(Literal::Number(array_pat.elements.len() as f64)),
                            HirExpr::Lit(Literal::Undefined),
                        ],
                    };
                    self.lower_pattern(&rest.argument, rest_expr, out)?;
                }
                Ok(())
            }
            BindingPattern::ObjectPattern(object_pat) => {
                for bp in &object_pat.properties {
                    let prop_name = self.prop_name_to_string(&bp.key);
                    let member_expr = HirExpr::MemberGet {
                        object: Box::new(base_expr.clone()),
                        property: prop_name.clone(),
                    };
                    
                    if bp.shorthand {
                        if let BindingPattern::AssignmentPattern(assign_pat) = &bp.value {
                            let default_val = self.lower_expr(&assign_pat.right)?;
                            let cond = HirExpr::BinOp(
                                BinOp::Eq,
                                Box::new(member_expr.clone()),
                                Box::new(HirExpr::Lit(Literal::Undefined)),
                            );
                            let val_expr = HirExpr::Ternary {
                                cond: Box::new(cond),
                                then_expr: Box::new(default_val),
                                else_expr: Box::new(member_expr),
                            };
                            self.lower_pattern(&assign_pat.left, val_expr, out)?;
                        } else {
                            self.lower_pattern(&bp.value, member_expr, out)?;
                        }
                    } else {
                        self.lower_pattern(&bp.value, member_expr, out)?;
                    }
                }
                // Handle rest element
                if let Some(rest_elem) = &object_pat.rest {
                    let mut extracted_keys = Vec::new();
                    for bp in &object_pat.properties {
                        extracted_keys.push(HirExpr::Lit(Literal::String(self.prop_name_to_string(&bp.key))));
                    }
                    let keys_array = HirExpr::ArrayLit(extracted_keys);
                    let rest_expr = HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_object_rest".to_string())),
                        args: vec![base_expr.clone(), keys_array],
                    };
                    self.lower_pattern(&rest_elem.argument, rest_expr, out)?;
                }
                Ok(())
            }
            BindingPattern::AssignmentPattern(assign_pat) => {
                let default_val = self.lower_expr(&assign_pat.right)?;
                let cond = HirExpr::BinOp(
                    BinOp::Eq,
                    Box::new(base_expr.clone()),
                    Box::new(HirExpr::Lit(Literal::Undefined)),
                );
                let final_expr = HirExpr::Ternary {
                    cond: Box::new(cond),
                    then_expr: Box::new(default_val),
                    else_expr: Box::new(base_expr),
                };
                self.lower_pattern(&assign_pat.left, final_expr, out)?;
                Ok(())
            }
        }
    }
}
