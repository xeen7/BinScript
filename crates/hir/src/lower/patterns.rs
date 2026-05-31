//! Pattern (destructuring) lowering.

use swc_core::ecma::ast::*;
use diagnostics::CompileResult;
use crate::types::*;
use super::context::LowerCtx;

impl LowerCtx {
    pub(crate) fn prop_name_to_string(&self, prop: &PropName) -> String {
        match prop {
            PropName::Ident(id) => id.sym.to_string(),
            PropName::Str(s) => s.value.as_wtf8().to_string_lossy().into_owned(),
            PropName::Num(n) => n.value.to_string(),
            _ => "unknown".to_string(),
        }
    }

    pub(crate) fn lower_pattern(&mut self, pat: &Pat, base_expr: HirExpr, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        match pat {
            Pat::Ident(binding_ident) => {
                let name = binding_ident.sym.to_string();
                let binding = self.declare(&name);
                out.push(HirStmt::Let {
                    binding,
                    name,
                    init: Some(base_expr),
                });
                Ok(())
            }
            Pat::Array(array_pat) => {
                for (idx, elem_opt) in array_pat.elems.iter().enumerate() {
                    if let Some(elem_pat) = elem_opt {
                        if let Pat::Rest(rest_pat) = elem_pat {
                            let rest_expr = HirExpr::MethodCall {
                                object: Box::new(base_expr.clone()),
                                method: "slice".to_string(),
                                args: vec![
                                    HirExpr::Lit(Literal::Number(idx as f64)),
                                    HirExpr::Lit(Literal::Undefined),
                                ],
                            };
                            self.lower_pattern(&rest_pat.arg, rest_expr, out)?;
                        } else {
                            let elem_expr = HirExpr::IndexGet {
                                object: Box::new(base_expr.clone()),
                                index: Box::new(HirExpr::Lit(Literal::Number(idx as f64))),
                            };
                            self.lower_pattern(elem_pat, elem_expr, out)?;
                        }
                    }
                }
                Ok(())
            }
            Pat::Object(object_pat) => {
                for prop in &object_pat.props {
                    match prop {
                        ObjectPatProp::Assign(assign_prop) => {
                            let prop_name = assign_prop.key.sym.to_string();
                            let member_expr = HirExpr::MemberGet {
                                object: Box::new(base_expr.clone()),
                                property: prop_name.clone(),
                            };
                            let val_expr = if let Some(default_expr) = &assign_prop.value {
                                let default_val = self.lower_expr(default_expr)?;
                                let cond = HirExpr::BinOp(
                                    BinOp::Eq,
                                    Box::new(member_expr.clone()),
                                    Box::new(HirExpr::Lit(Literal::Undefined)),
                                );
                                HirExpr::Ternary {
                                    cond: Box::new(cond),
                                    then_expr: Box::new(default_val),
                                    else_expr: Box::new(member_expr),
                                }
                            } else {
                                member_expr
                            };
                            let binding = self.declare(&prop_name);
                            out.push(HirStmt::Let {
                                binding,
                                name: prop_name,
                                init: Some(val_expr),
                            });
                        }
                        ObjectPatProp::KeyValue(kv_prop) => {
                            let prop_name = self.prop_name_to_string(&kv_prop.key);
                            let member_expr = HirExpr::MemberGet {
                                object: Box::new(base_expr.clone()),
                                property: prop_name,
                            };
                            self.lower_pattern(&kv_prop.value, member_expr, out)?;
                        }
                        ObjectPatProp::Rest(rest_pat) => {
                            let mut extracted_keys = Vec::new();
                            for other_prop in &object_pat.props {
                                match other_prop {
                                    ObjectPatProp::Assign(ap) => {
                                        extracted_keys.push(HirExpr::Lit(Literal::String(ap.key.sym.to_string())));
                                    }
                                    ObjectPatProp::KeyValue(kv) => {
                                        extracted_keys.push(HirExpr::Lit(Literal::String(self.prop_name_to_string(&kv.key))));
                                    }
                                    ObjectPatProp::Rest(_) => {}
                                }
                            }
                            let keys_array = HirExpr::ArrayLit(extracted_keys);
                            let rest_expr = HirExpr::Call {
                                callee: Box::new(HirExpr::GlobalRef("__bs_object_rest".to_string())),
                                args: vec![base_expr.clone(), keys_array],
                            };
                            self.lower_pattern(&rest_pat.arg, rest_expr, out)?;
                        }
                    }
                }
                Ok(())
            }
            Pat::Assign(assign_pat) => {
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
            Pat::Rest(rest_pat) => {
                // For function rest parameters (`...rest`), the caller is responsible for
                // bundling excess arguments into an array. Just bind the parameter directly.
                self.lower_pattern(&rest_pat.arg, base_expr, out)?;
                Ok(())
            }
            _ => Err(diagnostics::CompileError::Lowering {
                message: format!("Unsupported destructuring pattern type in lower_pattern: {:?}", pat),
            }),
        }
    }
}

