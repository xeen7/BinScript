use swc_core::ecma::ast::*;

use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::LowerCtx;
impl LowerCtx {
    pub(super) fn lower_expr_call(&mut self, call: &CallExpr) -> CompileResult<HirExpr> {
        let has_spread = call.args.iter().any(|a| a.spread.is_some());
        let is_method_call = match &call.callee {
            Callee::Expr(expr) => matches!(&**expr, Expr::Member(_)),
            _ => false,
        };
        if has_spread && !is_method_call {
            self.lower_call_with_spread(call)
        } else {
            self.lower_call(call)
        }
    }

    fn lower_call_with_spread(&mut self, call: &CallExpr) -> CompileResult<HirExpr> {
        let mut array_elems = Vec::new();
        for arg in &call.args {
            let expr = self.lower_expr(&arg.expr)?;
            if arg.spread.is_some() {
                array_elems.push(HirExpr::Spread(Box::new(expr)));
            } else {
                array_elems.push(expr);
            }
        }
        let args_array = HirExpr::ArrayLit(array_elems);

        match &call.callee {
            Callee::Super(_) => {
                Err(CompileError::Lowering {
                    message: "super() call with spread is not supported".into(),
                })
            }
            Callee::Expr(expr) => match &**expr {
                Expr::SuperProp(sp) => {
                    if let Some(super_c) = self.current_super.clone() {
                        if let Some(this_id) = self.this_binding {
                            let method_name = match &sp.prop {
                                SuperProp::Ident(prop_id) => prop_id.sym.to_string(),
                                SuperProp::Computed(_) => {
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
                    Err(CompileError::Lowering {
                        message: "super method call with spread outside class method".into(),
                    })
                }
                Expr::Member(m) => {
                    let obj_expr = self.lower_expr(&m.obj)?;
                    let method_name = match &m.prop {
                        MemberProp::Ident(prop_id) => prop_id.sym.to_string(),
                        MemberProp::PrivateName(pn) => format!("__private_{}", pn.name),
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
            _ => Err(CompileError::Lowering {
                message: "Unsupported callee type for spread call".into(),
            })
        }
    }

    fn lower_call(&mut self, call: &CallExpr) -> CompileResult<HirExpr> {
        let args: Vec<HirExpr> = call
            .args
            .iter()
            .map(|a| {
                let expr = self.lower_expr(&a.expr)?;
                if a.spread.is_some() {
                    Ok(HirExpr::Spread(Box::new(expr)))
                } else {
                    Ok(expr)
                }
            })
            .collect::<CompileResult<_>>()?;

        match &call.callee {
            Callee::Super(_) => {
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
                    for arg in &call.args {
                        c_args.push(self.lower_expr(&arg.expr)?);
                    }
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
            Callee::Expr(expr) => match &**expr {
                Expr::SuperProp(sp) => {
                    if let Some(super_c) = self.current_super.clone() {
                        if let Some(this_id) = self.this_binding {
                            match &sp.prop {
                                SuperProp::Ident(prop_id) => {
                                    let method_name = prop_id.sym.to_string();
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
                                SuperProp::Computed(_) => {
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
                // console.log(…), Math.floor(…), etc.
                Expr::Member(m) => {
                    if let (Expr::Ident(obj), MemberProp::Ident(prop)) = (&*m.obj, &m.prop) {
                        let obj_name = obj.sym.to_string();
                        let prop_name = prop.sym.to_string();
                        
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
                    
                    let obj_expr = self.lower_expr(&m.obj)?;
                    match &m.prop {
                        MemberProp::Ident(prop_id) => {
                            Ok(HirExpr::MethodCall {
                                object: Box::new(obj_expr),
                                method: prop_id.sym.to_string(),
                                args,
                            })
                        }
                        MemberProp::PrivateName(pn) => {
                            Ok(HirExpr::MethodCall {
                                object: Box::new(obj_expr),
                                method: format!("__private_{}", pn.name),
                                args,
                            })
                        }
                        MemberProp::Computed(computed) => {
                            let index_expr = self.lower_expr(&computed.expr)?;
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
                Expr::Ident(id) => {
                    let name = id.sym.to_string();
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
            },
            Callee::Import(_) => {
                if call.args.len() == 1 {
                    let specifier = self.lower_expr(&call.args[0].expr)?;
                    Ok(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_dynamic_import".to_string())),
                        args: vec![specifier],
                    })
                } else {
                    Err(CompileError::Lowering {
                        message: "Dynamic import expects exactly 1 argument".into(),
                    })
                }
            }
        }
    }
}
