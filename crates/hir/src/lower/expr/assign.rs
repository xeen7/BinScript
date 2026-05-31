use swc_core::ecma::ast::*;

use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_assign(&mut self, a: &AssignExpr) -> CompileResult<HirExpr> {
        let map_assign_op = |op: AssignOp| -> Option<BinOp> {
            match op {
                AssignOp::Assign => None,
                AssignOp::AddAssign => Some(BinOp::Add),
                AssignOp::SubAssign => Some(BinOp::Sub),
                AssignOp::MulAssign => Some(BinOp::Mul),
                AssignOp::DivAssign => Some(BinOp::Div),
                AssignOp::ModAssign => Some(BinOp::Mod),
                AssignOp::LShiftAssign => Some(BinOp::Shl),
                AssignOp::RShiftAssign => Some(BinOp::Shr),
                AssignOp::ZeroFillRShiftAssign => Some(BinOp::UShr),
                AssignOp::BitOrAssign => Some(BinOp::BitOr),
                AssignOp::BitXorAssign => Some(BinOp::BitXor),
                AssignOp::BitAndAssign => Some(BinOp::BitAnd),
                AssignOp::ExpAssign => Some(BinOp::Exp),
                AssignOp::AndAssign => Some(BinOp::And),
                AssignOp::OrAssign => Some(BinOp::Or),
                AssignOp::NullishAssign => Some(BinOp::NullishCoalescing),
            }
        };

        match &a.left {
            AssignTarget::Simple(SimpleAssignTarget::Ident(id)) => {
                let name = id.sym.to_string();
                let binding = self.lookup(&name).ok_or_else(|| CompileError::Lowering {
                    message: format!("Undefined variable in assignment: {}", name),
                })?;
                self.record_lookup(binding);
                self.reassigned_bindings.insert(binding);
                self.const_strings.remove(&binding);
                let mut value = match &*a.right {
                    Expr::Class(ce) if a.op == AssignOp::Assign => {
                        self.lower_expr_class_with_name(ce, name.clone())?
                    }
                    _ => self.lower_expr(&a.right)?,
                };
                
                if let Some(bin_op) = map_assign_op(a.op) {
                    value = HirExpr::BinOp(bin_op, Box::new(HirExpr::Var(binding)), Box::new(value));
                }

                Ok(HirExpr::Assign { target: binding, value: Box::new(value) })
            }
            AssignTarget::Simple(SimpleAssignTarget::Member(m)) => {
                let obj = self.lower_expr(&m.obj)?;
                let mut val = self.lower_expr(&a.right)?;
                match &m.prop {
                    MemberProp::Ident(prop_id) => {
                        let prop_name = prop_id.sym.to_string();
                        if let Some(bin_op) = map_assign_op(a.op) {
                            val = HirExpr::BinOp(
                                bin_op,
                                Box::new(HirExpr::MemberGet {
                                    object: Box::new(obj.clone()),
                                    property: prop_name.clone(),
                                }),
                                Box::new(val),
                            );
                        }
                        Ok(HirExpr::MemberSet {
                            object: Box::new(obj),
                            property: prop_name,
                            value: Box::new(val),
                        })
                    }
                    MemberProp::Computed(computed) => {
                        let idx = self.lower_expr(&computed.expr)?;
                        if let Some(bin_op) = map_assign_op(a.op) {
                            val = HirExpr::BinOp(
                                bin_op,
                                Box::new(HirExpr::IndexGet {
                                    object: Box::new(obj.clone()),
                                    index: Box::new(idx.clone()),
                                }),
                                Box::new(val),
                            );
                        }
                        Ok(HirExpr::IndexSet {
                            object: Box::new(obj),
                            index: Box::new(idx),
                            value: Box::new(val),
                        })
                    }
                    MemberProp::PrivateName(pn) => {
                        let prop_name = format!("__private_{}", pn.name);
                        if let Some(bin_op) = map_assign_op(a.op) {
                            val = HirExpr::BinOp(
                                bin_op,
                                Box::new(HirExpr::MemberGet {
                                    object: Box::new(obj.clone()),
                                    property: prop_name.clone(),
                                }),
                                Box::new(val),
                            );
                        }
                        Ok(HirExpr::MemberSet {
                            object: Box::new(obj),
                            property: prop_name,
                            value: Box::new(val),
                        })
                    }
                }
            }
            AssignTarget::Pat(pat) => {
                let base_pat = match pat {
                    AssignTargetPat::Array(array_pat) => Pat::Array(array_pat.clone()),
                    AssignTargetPat::Object(object_pat) => Pat::Object(object_pat.clone()),
                    _ => {
                        return Err(CompileError::Lowering {
                            message: "Invalid destructuring assignment target".into(),
                        });
                    }
                };

                let right_val = self.lower_expr(&a.right)?;
                let func_id = self.fresh_func_id();
                
                self.function_stack.push(func_id);
                self.push_scope();
                let temp_param_id = self.declare("_temp");
                
                let mut body = Vec::new();
                self.lower_assign_to_pattern(&base_pat, HirExpr::Var(temp_param_id), &mut body)?;
                
                body.push(HirStmt::Return(Some(HirExpr::Var(temp_param_id))));
                self.pop_scope();
                self.function_stack.pop();

                self.functions.push(HirFunction {
                    id: func_id,
                    name: format!("__bs_iife_destruct_{}", func_id),
                    params: vec![(temp_param_id, "_temp".to_string())],
                    body,
                    captures: Vec::new(),
                    is_generator: false,
                    is_async: false,
                });

                let closure = HirExpr::Closure { func_id, captures: Vec::new() };
                Ok(HirExpr::Call {
                    callee: Box::new(closure),
                    args: vec![right_val],
                })
            }
            _ => Err(CompileError::Lowering {
                message: "Complex assignment targets not yet supported".into(),
            }),
        }
    }

    fn lower_assign_to_pattern(&mut self, pat: &Pat, base_expr: HirExpr, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        match pat {
            Pat::Ident(binding_ident) => {
                let name = binding_ident.sym.to_string();
                let binding = self.lookup(&name).ok_or_else(|| CompileError::Lowering {
                    message: format!("Undefined variable in destructuring assignment: {}", name),
                })?;
                self.record_lookup(binding);
                self.reassigned_bindings.insert(binding);
                self.const_strings.remove(&binding);
                out.push(HirStmt::Expr(HirExpr::Assign {
                    target: binding,
                    value: Box::new(base_expr),
                }));
                Ok(())
            }
            Pat::Array(array_pat) => {
                let temp_name = format!("_temp_assign_{}", self.fresh_func_id());
                let temp_binding = self.declare(&temp_name);
                out.push(HirStmt::Let {
                    binding: temp_binding,
                    name: temp_name.clone(),
                    init: Some(base_expr),
                });
                let temp_expr = HirExpr::Var(temp_binding);

                for (idx, elem_opt) in array_pat.elems.iter().enumerate() {
                    if let Some(elem_pat) = elem_opt {
                        if let Pat::Rest(rest_pat) = elem_pat {
                            let rest_expr = HirExpr::MethodCall {
                                object: Box::new(temp_expr.clone()),
                                method: "slice".to_string(),
                                args: vec![
                                    HirExpr::Lit(Literal::Number(idx as f64)),
                                    HirExpr::Lit(Literal::Undefined),
                                ],
                            };
                            self.lower_assign_to_pattern(&rest_pat.arg, rest_expr, out)?;
                        } else {
                            let elem_expr = HirExpr::IndexGet {
                                object: Box::new(temp_expr.clone()),
                                index: Box::new(HirExpr::Lit(Literal::Number(idx as f64))),
                            };
                            self.lower_assign_to_pattern(elem_pat, elem_expr, out)?;
                        }
                    }
                }
                Ok(())
            }
            Pat::Object(object_pat) => {
                let temp_name = format!("_temp_assign_{}", self.fresh_func_id());
                let temp_binding = self.declare(&temp_name);
                out.push(HirStmt::Let {
                    binding: temp_binding,
                    name: temp_name.clone(),
                    init: Some(base_expr),
                });
                let temp_expr = HirExpr::Var(temp_binding);

                for prop in &object_pat.props {
                    match prop {
                        ObjectPatProp::Assign(assign_prop) => {
                            let prop_name = assign_prop.key.sym.to_string();
                            let member_expr = HirExpr::MemberGet {
                                object: Box::new(temp_expr.clone()),
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
                            let binding = self.lookup(&prop_name).ok_or_else(|| CompileError::Lowering {
                                message: format!("Undefined variable in destructuring assignment: {}", prop_name),
                            })?;
                            self.record_lookup(binding);
                            self.reassigned_bindings.insert(binding);
                            self.const_strings.remove(&binding);
                            out.push(HirStmt::Expr(HirExpr::Assign {
                                target: binding,
                                value: Box::new(val_expr),
                            }));
                        }
                        ObjectPatProp::KeyValue(kv_prop) => {
                            let prop_name = self.prop_name_to_string(&kv_prop.key);
                            let member_expr = HirExpr::MemberGet {
                                object: Box::new(temp_expr.clone()),
                                property: prop_name,
                            };
                            self.lower_assign_to_pattern(&kv_prop.value, member_expr, out)?;
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
                                args: vec![temp_expr.clone(), keys_array],
                            };
                            self.lower_assign_to_pattern(&rest_pat.arg, rest_expr, out)?;
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
                self.lower_assign_to_pattern(&assign_pat.left, final_expr, out)?;
                Ok(())
            }
            Pat::Expr(expr) => {
                let val_expr = base_expr;
                match &**expr {
                    Expr::Ident(id) => {
                        let name = id.sym.to_string();
                        let binding = self.lookup(&name).ok_or_else(|| CompileError::Lowering {
                            message: format!("Undefined variable in assignment: {}", name),
                        })?;
                        self.record_lookup(binding);
                        self.reassigned_bindings.insert(binding);
                        self.const_strings.remove(&binding);
                        out.push(HirStmt::Expr(HirExpr::Assign {
                            target: binding,
                            value: Box::new(val_expr),
                        }));
                        Ok(())
                    }
                    Expr::Member(m) => {
                        let obj = self.lower_expr(&m.obj)?;
                        match &m.prop {
                            MemberProp::Ident(prop_id) => {
                                let prop_name = prop_id.sym.to_string();
                                out.push(HirStmt::Expr(HirExpr::MemberSet {
                                    object: Box::new(obj),
                                    property: prop_name,
                                    value: Box::new(val_expr),
                                }));
                                Ok(())
                            }
                            MemberProp::Computed(computed) => {
                                let idx = self.lower_expr(&computed.expr)?;
                                out.push(HirStmt::Expr(HirExpr::IndexSet {
                                    object: Box::new(obj),
                                    index: Box::new(idx),
                                    value: Box::new(val_expr),
                                }));
                                Ok(())
                            }
                            MemberProp::PrivateName(pn) => {
                                let prop_name = format!("__private_{}", pn.name);
                                out.push(HirStmt::Expr(HirExpr::MemberSet {
                                    object: Box::new(obj),
                                    property: prop_name,
                                    value: Box::new(val_expr),
                                }));
                                Ok(())
                            }
                        }
                    }
                    _ => Err(CompileError::Lowering {
                        message: "Invalid destructuring assignment target expression".into(),
                    }),
                }
            }
            _ => Err(CompileError::Lowering {
                message: format!("Unsupported destructuring pattern type in assign: {:?}", pat),
            }),
        }
    }
}
