use oxc::ast::ast::*;

use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_assign(&mut self, a: &AssignmentExpression) -> CompileResult<HirExpr> {
        let map_assign_op = |op: AssignmentOperator| -> Option<BinOp> {
            match op {
                AssignmentOperator::Assign => None,
                AssignmentOperator::Addition => Some(BinOp::Add),
                AssignmentOperator::Subtraction => Some(BinOp::Sub),
                AssignmentOperator::Multiplication => Some(BinOp::Mul),
                AssignmentOperator::Division => Some(BinOp::Div),
                AssignmentOperator::Remainder => Some(BinOp::Mod),
                AssignmentOperator::ShiftLeft => Some(BinOp::Shl),
                AssignmentOperator::ShiftRight => Some(BinOp::Shr),
                AssignmentOperator::ShiftRightZeroFill => Some(BinOp::UShr),
                AssignmentOperator::BitwiseOR => Some(BinOp::BitOr),
                AssignmentOperator::BitwiseXOR => Some(BinOp::BitXor),
                AssignmentOperator::BitwiseAnd => Some(BinOp::BitAnd),
                AssignmentOperator::Exponential => Some(BinOp::Exp),
                AssignmentOperator::LogicalAnd => Some(BinOp::And),
                AssignmentOperator::LogicalOr => Some(BinOp::Or),
                AssignmentOperator::LogicalNullish => Some(BinOp::NullishCoalescing),
            }
        };

        // Handle identifier assignment: x = ...
        if let AssignmentTarget::AssignmentTargetIdentifier(id) = &a.left {
            let name = id.name.to_string();
            let binding = self.lookup(&name).ok_or_else(|| CompileError::Lowering {
                message: format!("Undefined variable in assignment: {}", name),
            })?;
            self.record_lookup(binding);
            self.reassigned_bindings.insert(binding);
            self.const_strings.remove(&binding);
            let mut value = match &a.right {
                Expression::ClassExpression(ce) if a.operator == AssignmentOperator::Assign => {
                    self.lower_expr_class_with_name(ce, name.clone())?
                }
                _ => self.lower_expr(&a.right)?,
            };
            
            if let Some(bin_op) = map_assign_op(a.operator) {
                if matches!(bin_op, BinOp::And | BinOp::Or | BinOp::NullishCoalescing) {
                    // Short-circuiting assignment for simple variable: x &&= b -> x && (x = b)
                    return Ok(HirExpr::BinOp(
                        bin_op,
                        Box::new(HirExpr::Var(binding)),
                        Box::new(HirExpr::Assign { target: binding, value: Box::new(value) })
                    ));
                }
                value = HirExpr::BinOp(bin_op, Box::new(HirExpr::Var(binding)), Box::new(value));
            }

            return Ok(HirExpr::Assign { target: binding, value: Box::new(value) });
        }

        // Handle member expression assignment: obj.prop = ... or obj[idx] = ...
        if let Some(m) = a.left.as_member_expression() {
            let obj = self.lower_expr(m.object())?;
            let mut val = self.lower_expr(&a.right)?;
            return match m {
                MemberExpression::StaticMemberExpression(prop) => {
                    let prop_name = prop.property.name.to_string();
                    if let Some(bin_op) = map_assign_op(a.operator) {
                        return Ok(HirExpr::CompoundMemberSet {
                            object: Box::new(obj),
                            property: prop_name,
                            op: bin_op,
                            value: Box::new(val),
                        });
                    }
                    Ok(HirExpr::MemberSet {
                        object: Box::new(obj),
                        property: prop_name,
                        value: Box::new(val),
                    })
                }
                MemberExpression::ComputedMemberExpression(computed) => {
                    let idx = self.lower_expr(&computed.expression)?;
                    if let Some(bin_op) = map_assign_op(a.operator) {
                        return Ok(HirExpr::CompoundIndexSet {
                            object: Box::new(obj),
                            index: Box::new(idx),
                            op: bin_op,
                            value: Box::new(val),
                        });
                    }
                    Ok(HirExpr::IndexSet {
                        object: Box::new(obj),
                        index: Box::new(idx),
                        value: Box::new(val),
                    })
                }
                MemberExpression::PrivateFieldExpression(pn) => {
                    let prop_name = format!("__private_{}", pn.field.name);
                    if let Some(bin_op) = map_assign_op(a.operator) {
                        return Ok(HirExpr::CompoundMemberSet {
                            object: Box::new(obj),
                            property: prop_name,
                            op: bin_op,
                            value: Box::new(val),
                        });
                    }
                    Ok(HirExpr::MemberSet {
                        object: Box::new(obj),
                        property: prop_name,
                        value: Box::new(val),
                    })
                }
            };
        }

        // Handle destructuring assignment: [a, b] = ... or {a, b} = ...
        if let Some(pat) = a.left.as_assignment_target_pattern() {
            let right_val = self.lower_expr(&a.right)?;
            let func_id = self.fresh_func_id();
            
            self.function_stack.push(func_id);
            self.push_scope();
            let temp_param_id = self.declare("_temp");
            
            let mut body = Vec::new();
            self.lower_assign_to_pattern(pat, HirExpr::Var(temp_param_id), &mut body)?;
            
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
            return Ok(HirExpr::Call {
                callee: Box::new(closure),
                args: vec![right_val],
            });
        }

        Err(CompileError::Lowering {
            message: "Complex assignment targets not yet supported".into(),
        })
    }

    fn lower_assign_to_pattern(&mut self, pat: &AssignmentTargetPattern, base_expr: HirExpr, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        match pat {
            AssignmentTargetPattern::ArrayAssignmentTarget(array_pat) => {
                let temp_name = format!("_temp_assign_{}", self.fresh_func_id());
                let temp_binding = self.declare(&temp_name);
                out.push(HirStmt::Let {
                    binding: temp_binding,
                    name: temp_name.clone(),
                    init: Some(base_expr),
                });
                let temp_expr = HirExpr::Var(temp_binding);

                for (idx, elem_opt) in array_pat.elements.iter().enumerate() {
                    if let Some(elem_pat) = elem_opt {
                        let elem_expr = HirExpr::IndexGet {
                            object: Box::new(temp_expr.clone()),
                            index: Box::new(HirExpr::Lit(Literal::Number(idx as f64))),
                        };
                        if let AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(assign_pat) = elem_pat {
                            let default_val = self.lower_expr(&assign_pat.init)?;
                            let cond = HirExpr::BinOp(
                                BinOp::Eq,
                                Box::new(elem_expr.clone()),
                                Box::new(HirExpr::Lit(Literal::Undefined)),
                            );
                            let final_expr = HirExpr::Ternary {
                                cond: Box::new(cond),
                                then_expr: Box::new(default_val),
                                else_expr: Box::new(elem_expr),
                            };
                            self.lower_assignment_target(&assign_pat.binding, final_expr, out)?;
                        } else if let Some(target) = elem_pat.as_assignment_target() {
                            self.lower_assignment_target(target, elem_expr, out)?;
                        }
                    }
                }
                
                if let Some(rest) = &array_pat.rest {
                    let rest_expr = HirExpr::MethodCall {
                        object: Box::new(temp_expr.clone()),
                        method: "slice".to_string(),
                        args: vec![
                            HirExpr::Lit(Literal::Number(array_pat.elements.len() as f64)),
                            HirExpr::Lit(Literal::Undefined),
                        ],
                    };
                    self.lower_assignment_target(&rest.target, rest_expr, out)?;
                }
                Ok(())
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(object_pat) => {
                let temp_name = format!("_temp_assign_{}", self.fresh_func_id());
                let temp_binding = self.declare(&temp_name);
                out.push(HirStmt::Let {
                    binding: temp_binding,
                    name: temp_name.clone(),
                    init: Some(base_expr),
                });
                let temp_expr = HirExpr::Var(temp_binding);

                for prop in &object_pat.properties {
                    match prop {
                        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(assign_prop) => {
                            let prop_name = assign_prop.binding.name.to_string();
                            let member_expr = HirExpr::MemberGet {
                                object: Box::new(temp_expr.clone()),
                                property: prop_name.clone(),
                            };
                            let val_expr = if let Some(default_expr) = &assign_prop.init {
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
                        AssignmentTargetProperty::AssignmentTargetPropertyProperty(kv_prop) => {
                            let prop_name = match &kv_prop.name {
                                PropertyKey::StaticIdentifier(id) => id.name.to_string(),
                                PropertyKey::StringLiteral(s) => s.value.to_string(),
                                PropertyKey::NumericLiteral(n) => n.value.to_string(),
                                _ => return Err(CompileError::Lowering {
                                    message: "Unsupported property key in object assignment target".into(),
                                }),
                            };
                            let member_expr = HirExpr::MemberGet {
                                object: Box::new(temp_expr.clone()),
                                property: prop_name,
                            };
                            
                            if let AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(assign_pat) = &kv_prop.binding {
                                let default_val = self.lower_expr(&assign_pat.init)?;
                                let cond = HirExpr::BinOp(
                                    BinOp::Eq,
                                    Box::new(member_expr.clone()),
                                    Box::new(HirExpr::Lit(Literal::Undefined)),
                                );
                                let final_expr = HirExpr::Ternary {
                                    cond: Box::new(cond),
                                    then_expr: Box::new(default_val),
                                    else_expr: Box::new(member_expr),
                                };
                                self.lower_assignment_target(&assign_pat.binding, final_expr, out)?;
                            } else if let Some(target) = kv_prop.binding.as_assignment_target() {
                                self.lower_assignment_target(target, member_expr, out)?;
                            }
                        }
                    }
                }
                
                if let Some(rest) = &object_pat.rest {
                    let mut extracted_keys = Vec::new();
                    for other_prop in &object_pat.properties {
                        match other_prop {
                            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(ap) => {
                                extracted_keys.push(HirExpr::Lit(Literal::String(ap.binding.name.to_string())));
                            }
                            AssignmentTargetProperty::AssignmentTargetPropertyProperty(kv) => {
                                let key_str = match &kv.name {
                                    PropertyKey::StaticIdentifier(id) => id.name.to_string(),
                                    PropertyKey::StringLiteral(s) => s.value.to_string(),
                                    PropertyKey::NumericLiteral(n) => n.value.to_string(),
                                    _ => "".to_string(),
                                };
                                extracted_keys.push(HirExpr::Lit(Literal::String(key_str)));
                            }
                        }
                    }
                    let keys_array = HirExpr::ArrayLit(extracted_keys);
                    let rest_expr = HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_object_rest".to_string())),
                        args: vec![temp_expr.clone(), keys_array],
                    };
                    self.lower_assignment_target(&rest.target, rest_expr, out)?;
                }
                Ok(())
            }
        }
    }

    fn lower_assignment_target(&mut self, target: &AssignmentTarget, base_expr: HirExpr, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        if let Some(pat) = target.as_assignment_target_pattern() {
            return self.lower_assign_to_pattern(pat, base_expr, out);
        }

        if let AssignmentTarget::AssignmentTargetIdentifier(id) = target {
            let name = id.name.to_string();
            let binding = self.lookup(&name).ok_or_else(|| CompileError::Lowering {
                message: format!("Undefined variable in assignment: {}", name),
            })?;
            self.record_lookup(binding);
            self.reassigned_bindings.insert(binding);
            self.const_strings.remove(&binding);
            out.push(HirStmt::Expr(HirExpr::Assign {
                target: binding,
                value: Box::new(base_expr),
            }));
            return Ok(());
        }

        if let Some(m) = target.as_member_expression() {
            let obj = self.lower_expr(m.object())?;
            match m {
                MemberExpression::StaticMemberExpression(prop) => {
                    out.push(HirStmt::Expr(HirExpr::MemberSet {
                        object: Box::new(obj),
                        property: prop.property.name.to_string(),
                        value: Box::new(base_expr),
                    }));
                    Ok(())
                }
                MemberExpression::ComputedMemberExpression(computed) => {
                    let idx = self.lower_expr(&computed.expression)?;
                    out.push(HirStmt::Expr(HirExpr::IndexSet {
                        object: Box::new(obj),
                        index: Box::new(idx),
                        value: Box::new(base_expr),
                    }));
                    Ok(())
                }
                MemberExpression::PrivateFieldExpression(pn) => {
                    out.push(HirStmt::Expr(HirExpr::MemberSet {
                        object: Box::new(obj),
                        property: format!("__private_{}", pn.field.name),
                        value: Box::new(base_expr),
                    }));
                    Ok(())
                }
            }
        } else {
            Err(CompileError::Lowering {
                message: "Invalid destructuring assignment target expression".into(),
            })
        }
    }
}
