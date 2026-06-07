use oxc::ast::ast::*;

use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_call(&mut self, call: &CallExpression) -> CompileResult<HirExpr> {
        let has_spread = call.arguments.iter().any(|a| matches!(a, Argument::SpreadElement(_)));
        let is_method_call = call.callee.is_member_expression();
        if has_spread && !is_method_call {
            self.lower_call_with_spread(call)
        } else {
            self.lower_call(call)
        }
    }

    fn lower_call_with_spread(&mut self, call: &CallExpression) -> CompileResult<HirExpr> {
        let mut array_elems = Vec::new();
        for arg in &call.arguments {
            match arg {
                Argument::SpreadElement(spread) => {
                    let expr = self.lower_expr(&spread.argument)?;
                    array_elems.push(HirExpr::Spread(Box::new(expr)));
                }
                other => {
                    let expr = other.as_expression().unwrap();
                    let e = self.lower_expr(expr)?;
                    array_elems.push(e);
                }
            }
        }
        let args_array = HirExpr::ArrayLit(array_elems);

        match &call.callee {
            Expression::Super(_) => {
                Err(CompileError::Lowering {
                    message: "super() call with spread is not supported".into(),
                })
            }
            m if m.is_member_expression() => {
                let m = m.as_member_expression().unwrap();
                if matches!(m.object(), Expression::Super(_)) {
                    if let Some(super_c) = self.current_super.clone() {
                        if let Some(this_id) = self.this_binding {
                            let method_name = match m {
                                MemberExpression::StaticMemberExpression(prop) => prop.property.name.to_string(),
                                _ => {
                                    return Err(CompileError::Lowering {
                                        message: "Dynamic/computed super method calls are not supported".into(),
                                    });
                                }
                            };
                            let callee_expr = HirExpr::GlobalRef(format!(
                                "__bs_class_{}_{}",
                                super_c, method_name
                            ));
                            
                            let mut resolved_elems = vec![HirExpr::Var(this_id)];
                            if let HirExpr::ArrayLit(mut elems) = args_array {
                                resolved_elems.append(&mut elems);
                            }
                            let resolved_args_array = HirExpr::ArrayLit(resolved_elems);
                            
                            return Ok(HirExpr::Call {
                                callee: Box::new(HirExpr::GlobalRef("__bs_call_apply".to_string())),
                                args: vec![callee_expr, HirExpr::Lit(Literal::Undefined), resolved_args_array],
                            });
                        }
                    }
                    return Err(CompileError::Lowering {
                        message: "super method call with spread outside class method".into(),
                    });
                }

                let obj_expr = self.lower_expr(m.object())?;
                let method_name = match m {
                    MemberExpression::StaticMemberExpression(s) => s.property.name.to_string(),
                    MemberExpression::PrivateFieldExpression(pn) => format!("__private_{}", pn.field.name),
                    _ => return Err(CompileError::Lowering {
                        message: "Computed dynamic method calls with spread not supported".into(),
                    }),
                };

                let temp_bid = self.next_binding;
                self.next_binding += 1;
                let current_func = *self.function_stack.last().unwrap_or(&0);
                self.binding_owners.insert(temp_bid, current_func);

                let assign_obj = HirExpr::Assign {
                    target: temp_bid,
                    value: Box::new(obj_expr),
                };

                let callee_expr = HirExpr::MemberGet {
                    object: Box::new(HirExpr::Var(temp_bid)),
                    property: method_name,
                };

                let mut resolved_elems = vec![HirExpr::Var(temp_bid)];
                if let HirExpr::ArrayLit(mut elems) = args_array {
                    resolved_elems.append(&mut elems);
                }
                let resolved_args_array = HirExpr::ArrayLit(resolved_elems);

                let call_apply = HirExpr::Call {
                    callee: Box::new(HirExpr::GlobalRef("__bs_call_apply".to_string())),
                    args: vec![callee_expr, HirExpr::Lit(Literal::Undefined), resolved_args_array],
                };

                Ok(HirExpr::Seq(vec![assign_obj, call_apply]))
            }
            other => {
                let callee_hir = self.lower_expr(other)?;
                Ok(HirExpr::Call {
                    callee: Box::new(HirExpr::GlobalRef("__bs_call_apply".to_string())),
                    args: vec![callee_hir, HirExpr::Lit(Literal::Undefined), args_array],
                })
            }
        }
    }

    fn lower_call(&mut self, call: &CallExpression) -> CompileResult<HirExpr> {
        let mut args = Vec::new();
        for arg in &call.arguments {
            match arg {
                Argument::SpreadElement(spread) => {
                    let expr = self.lower_expr(&spread.argument)?;
                    args.push(HirExpr::Spread(Box::new(expr)));
                }
                other => {
                    let expr = other.as_expression().unwrap();
                    let e = self.lower_expr(expr)?;
                    args.push(e);
                }
            }
        }

        match &call.callee {
            Expression::Super(_) => {
                if let Some(super_c) = self.current_super.clone() {
                    let mut c_args = Vec::new();
                    // Prepend `this`
                    if let Some(this_id) = self.this_binding {
                        c_args.push(HirExpr::Var(this_id));
                    } else {
                        return Err(CompileError::Lowering {
                            message: "super() called outside constructor".into(),
                        });
                    }
                    c_args.extend(args);
                    Ok(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef(format!("__bs_class_{}_constructor", super_c))),
                        args: c_args,
                    })
                } else {
                    Err(CompileError::Lowering {
                        message: "super() called in class with no parent".into(),
                    })
                }
            }
            m if m.is_member_expression() => {
                let m = m.as_member_expression().unwrap();
                if matches!(m.object(), Expression::Super(_)) {
                    if let Some(super_c) = self.current_super.clone() {
                        if let Some(this_id) = self.this_binding {
                            match m {
                                MemberExpression::StaticMemberExpression(prop) => {
                                    let method_name = prop.property.name.to_string();
                                    let mut resolved_args = vec![HirExpr::Var(this_id)];
                                    resolved_args.extend(args);
                                    return Ok(HirExpr::Call {
                                        callee: Box::new(HirExpr::GlobalRef(format!(
                                            "__bs_class_{}_{}",
                                            super_c, method_name
                                        ))),
                                        args: resolved_args,
                                    });
                                }
                                MemberExpression::ComputedMemberExpression(_) | MemberExpression::PrivateFieldExpression(_) => {
                                    return Err(CompileError::Lowering {
                                        message: "Dynamic/computed super method calls are not supported".into(),
                                    });
                                }
                            }
                        } else {
                            return Err(CompileError::Lowering {
                                message: "super method call outside class method".into(),
                            });
                        }
                    } else {
                        return Err(CompileError::Lowering {
                            message: "super method call in class with no parent".into(),
                        });
                    }
                }

                if let (Expression::Identifier(obj), MemberExpression::StaticMemberExpression(prop)) = (m.object(), m) {
                    let obj_name = obj.name.to_string();
                    let prop_name = prop.property.name.to_string();
                    
                    if obj_name == "JSON" && prop_name == "parse" {
                        if args.len() == 1 {
                            let mut resolved_str = None;
                            match &args[0] {
                                HirExpr::Lit(Literal::String(s)) => {
                                    resolved_str = Some(s.clone());
                                }
                                HirExpr::Var(bid) => {
                                    if let Some(s) = self.const_strings.get(bid) {
                                        resolved_str = Some(s.clone());
                                    }
                                }
                                _ => {}
                            }
                            if let Some(s) = resolved_str {
                                return Ok(HirExpr::JsonTape(std::sync::Arc::from(s.as_bytes())));
                            }
                        }
                    }

                    if obj_name == "console" || obj_name == "Math" || obj_name == "Promise" || obj_name == "JSON" || obj_name == "Number" || obj_name == "Object" || obj_name == "String" || obj_name == "Date" {
                        return Ok(HirExpr::MemberCall {
                            object: obj_name,
                            method: prop_name,
                            args,
                        });
                    }
                }
                
                let obj_expr = self.lower_expr(m.object())?;
                match m {
                    MemberExpression::StaticMemberExpression(prop) => {
                        Ok(HirExpr::MethodCall {
                            object: Box::new(obj_expr),
                            method: prop.property.name.to_string(),
                            args,
                        })
                    }
                    MemberExpression::PrivateFieldExpression(pn) => {
                        Ok(HirExpr::MethodCall {
                            object: Box::new(obj_expr),
                            method: format!("__private_{}", pn.field.name),
                            args,
                        })
                    }
                    MemberExpression::ComputedMemberExpression(computed) => {
                        let index_expr = self.lower_expr(&computed.expression)?;
                        Ok(HirExpr::Call {
                            callee: Box::new(HirExpr::IndexGet {
                                object: Box::new(obj_expr),
                                index: Box::new(index_expr),
                            }),
                            args,
                        })
                    }
                }
            }
            // foo(…)
            Expression::Identifier(id) => {
                let name = id.name.to_string();
                if self.lookup(&name).is_none() {
                    if name == "String" {
                        if args.is_empty() {
                            return Ok(HirExpr::Lit(Literal::String("".to_string())));
                        } else {
                            return Ok(HirExpr::Call {
                                callee: Box::new(HirExpr::GlobalRef("__bs_String".to_string())),
                                args: vec![args[0].clone()],
                            });
                        }
                    } else if name == "Number" {
                        if args.is_empty() {
                            return Ok(HirExpr::Lit(Literal::Number(0.0)));
                        } else {
                            return Ok(HirExpr::Call {
                                callee: Box::new(HirExpr::GlobalRef("__bs_Number".to_string())),
                                args: vec![args[0].clone()],
                            });
                        }
                    } else if name == "Boolean" {
                        if args.is_empty() {
                            return Ok(HirExpr::Lit(Literal::Bool(false)));
                        } else {
                            return Ok(HirExpr::Call {
                                callee: Box::new(HirExpr::GlobalRef("__bs_Boolean".to_string())),
                                args: vec![args[0].clone()],
                            });
                        }
                    } else if name == "Object" {
                        if args.is_empty() {
                            return Ok(HirExpr::Call {
                                callee: Box::new(HirExpr::GlobalRef("__bs_Object".to_string())),
                                args: vec![HirExpr::Lit(Literal::Undefined)],
                            });
                        } else {
                            return Ok(HirExpr::Call {
                                callee: Box::new(HirExpr::GlobalRef("__bs_Object".to_string())),
                                args: vec![args[0].clone()],
                            });
                        }
                    } else if name == "Date" {
                        return Ok(HirExpr::Call {
                            callee: Box::new(HirExpr::GlobalRef("__bs_Date".to_string())),
                            args: vec![HirExpr::Lit(Literal::Undefined)],
                        });
                    }
                }

                let callee = if self.function_names.contains(&name) {
                    HirExpr::GlobalRef(name)
                } else if let Some(aliased) = self.function_aliases.get(&name) {
                    HirExpr::GlobalRef(aliased.clone())
                } else {
                    self.lookup(&name)
                        .map(HirExpr::Var)
                        .unwrap_or_else(|| HirExpr::GlobalRef(name))
                };
                Ok(HirExpr::Call {
                    callee: Box::new(callee),
                    args,
                })
            }
            other => {
                let callee = self.lower_expr(other)?;
                Ok(HirExpr::Call { callee: Box::new(callee), args })
            }
        }
    }
}
