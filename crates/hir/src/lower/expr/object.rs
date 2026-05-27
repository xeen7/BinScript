use swc_core::ecma::ast::*;

use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_object(&mut self, obj: &ObjectLit) -> CompileResult<HirExpr> {
        let temp_bid = self.next_binding;
        self.next_binding += 1;
        let current_func = *self.function_stack.last().unwrap_or(&0);
        self.binding_owners.insert(temp_bid, current_func);
        
        let new_obj = HirExpr::Call {
            callee: Box::new(HirExpr::GlobalRef("__bs_new_object".to_string())),
            args: Vec::new(),
        };
        
        let assign_obj = HirExpr::Assign {
            target: temp_bid,
            value: Box::new(new_obj),
        };
        
        let mut seq = vec![assign_obj];
        
        for prop_or_spread in &obj.props {
            match prop_or_spread {
                PropOrSpread::Prop(p) => match &**p {
                    Prop::KeyValue(kv) => {
                        let val_expr = self.lower_expr(&kv.value)?;
                        match &kv.key {
                            PropName::Ident(id) => {
                                let key_name = id.sym.to_string();
                                seq.push(HirExpr::MemberSet {
                                    object: Box::new(HirExpr::Var(temp_bid)),
                                    property: key_name,
                                    value: Box::new(val_expr),
                                });
                            }
                            PropName::Str(s) => {
                                let key_name = s.value.as_wtf8().to_string_lossy().into_owned();
                                seq.push(HirExpr::MemberSet {
                                    object: Box::new(HirExpr::Var(temp_bid)),
                                    property: key_name,
                                    value: Box::new(val_expr),
                                });
                            }
                            PropName::Computed(comp) => {
                                let key_expr = self.lower_expr(&comp.expr)?;
                                seq.push(HirExpr::IndexSet {
                                    object: Box::new(HirExpr::Var(temp_bid)),
                                    index: Box::new(key_expr),
                                    value: Box::new(val_expr),
                                });
                            }
                            _ => return Err(CompileError::Lowering {
                                message: "Unsupported object property key type".into(),
                            }),
                        }
                    }
                    Prop::Shorthand(id) => {
                        let key_name = id.sym.to_string();
                        let val_expr = self.lower_expr(&Expr::Ident(id.clone()))?;
                        seq.push(HirExpr::MemberSet {
                            object: Box::new(HirExpr::Var(temp_bid)),
                            property: key_name,
                            value: Box::new(val_expr),
                        });
                    }
                    Prop::Method(method) => {
                        let val_expr = self.lower_function(&method.function, "anonymous_method".to_string())?;
                        match &method.key {
                            PropName::Ident(id) => {
                                let key_name = id.sym.to_string();
                                seq.push(HirExpr::MemberSet {
                                    object: Box::new(HirExpr::Var(temp_bid)),
                                    property: key_name,
                                    value: Box::new(val_expr),
                                });
                            }
                            PropName::Str(s) => {
                                let key_name = s.value.as_wtf8().to_string_lossy().into_owned();
                                seq.push(HirExpr::MemberSet {
                                    object: Box::new(HirExpr::Var(temp_bid)),
                                    property: key_name,
                                    value: Box::new(val_expr),
                                });
                            }
                            PropName::Computed(comp) => {
                                let key_expr = self.lower_expr(&comp.expr)?;
                                seq.push(HirExpr::IndexSet {
                                    object: Box::new(HirExpr::Var(temp_bid)),
                                    index: Box::new(key_expr),
                                    value: Box::new(val_expr),
                                });
                            }
                            _ => return Err(CompileError::Lowering {
                                message: "Unsupported computed key in method".into(),
                            }),
                        }
                    }
                    _ => return Err(CompileError::Lowering {
                        message: "Unsupported property type in object literal".into(),
                    }),
                }
                PropOrSpread::Spread(spread) => {
                    let spread_expr = self.lower_expr(&spread.expr)?;
                    seq.push(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_object_spread".to_string())),
                        args: vec![HirExpr::Var(temp_bid), spread_expr],
                    });
                }
            }
        }
        
        seq.push(HirExpr::Var(temp_bid));
        Ok(HirExpr::Seq(seq))
    }
}
