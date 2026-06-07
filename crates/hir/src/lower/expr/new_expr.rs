use oxc::ast::ast::*;

use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_new(&mut self, n: &NewExpression) -> CompileResult<HirExpr> {
        if let Expression::Identifier(id) = &n.callee {
            let mut args = Vec::new();
            for arg in &n.arguments {
                match arg {
                    Argument::SpreadElement(spread) => {
                        let e = self.lower_expr(&spread.argument)?;
                        args.push(HirExpr::Spread(Box::new(e)));
                    }
                    other => {
                        let expr = other.as_expression().unwrap();
                        args.push(self.lower_expr(expr)?);
                    }
                }
            }
            let raw_class_name = id.name.to_string();
            let class_name = self.class_aliases.get(&raw_class_name)
                .cloned()
                .unwrap_or(raw_class_name);
                
            if class_name == "Object" {
                if args.is_empty() {
                    return Ok(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_Object_new_0".to_string())),
                        args: vec![],
                    });
                } else {
                    return Ok(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_Object_new_1".to_string())),
                        args: vec![args[0].clone()],
                    });
                }
            }
            if class_name == "Array" {
                if args.len() > 1 {
                    return Ok(HirExpr::ArrayLit(args));
                } else {
                    let arg = if args.is_empty() {
                        HirExpr::Lit(Literal::Undefined)
                    } else {
                        args[0].clone()
                    };
                    return Ok(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_Array_new".to_string())),
                        args: vec![arg],
                    });
                }
            }
            if class_name == "String" {
                if args.is_empty() {
                    return Ok(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_String_new_0".to_string())),
                        args: vec![],
                    });
                } else {
                    return Ok(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_String_new_1".to_string())),
                        args: vec![args[0].clone()],
                    });
                }
            }
            if class_name == "Number" {
                if args.is_empty() {
                    return Ok(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_Number_new_0".to_string())),
                        args: vec![],
                    });
                } else {
                    return Ok(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_Number_new_1".to_string())),
                        args: vec![args[0].clone()],
                    });
                }
            }
            if class_name == "Boolean" {
                if args.is_empty() {
                    return Ok(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_Boolean_new_0".to_string())),
                        args: vec![],
                    });
                } else {
                    return Ok(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_Boolean_new_1".to_string())),
                        args: vec![args[0].clone()],
                    });
                }
            }
            if class_name == "Date" {
                if args.is_empty() {
                    return Ok(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_Date_new_0".to_string())),
                        args: vec![],
                    });
                } else if args.len() == 1 {
                    return Ok(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_Date_new_1".to_string())),
                        args: vec![args[0].clone()],
                    });
                } else {
                    let mut call_args = args.clone();
                    while call_args.len() < 7 {
                        call_args.push(HirExpr::Lit(Literal::Undefined));
                    }
                    if call_args.len() > 7 {
                        call_args.truncate(7);
                    }
                    return Ok(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_Date_new_n".to_string())),
                        args: call_args,
                    });
                }
            }
            if class_name == "Map" {
                if args.is_empty() {
                    return Ok(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_Map_new_0".to_string())),
                        args: vec![],
                    });
                } else {
                    return Ok(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_Map_new_1".to_string())),
                        args: vec![args[0].clone()],
                    });
                }
            }
            if class_name == "Set" {
                if args.is_empty() {
                    return Ok(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_Set_new_0".to_string())),
                        args: vec![],
                    });
                } else {
                    return Ok(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_Set_new_1".to_string())),
                        args: vec![args[0].clone()],
                    });
                }
            }
            if class_name == "WeakMap" {
                if args.is_empty() {
                    return Ok(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_WeakMap_new_0".to_string())),
                        args: vec![],
                    });
                } else {
                    return Ok(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_WeakMap_new_1".to_string())),
                        args: vec![args[0].clone()],
                    });
                }
            }
            if class_name == "WeakSet" {
                if args.is_empty() {
                    return Ok(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_WeakSet_new_0".to_string())),
                        args: vec![],
                    });
                } else {
                    return Ok(HirExpr::Call {
                        callee: Box::new(HirExpr::GlobalRef("__bs_WeakSet_new_1".to_string())),
                        args: vec![args[0].clone()],
                    });
                }
            }
            if class_name == "RegExp" {
                let pattern = if args.is_empty() {
                    HirExpr::Lit(Literal::String("".to_string()))
                } else {
                    args[0].clone()
                };
                let flags = if args.len() < 2 {
                    HirExpr::Lit(Literal::String("".to_string()))
                } else {
                    args[1].clone()
                };
                return Ok(HirExpr::Call {
                    callee: Box::new(HirExpr::GlobalRef("__bs_RegExp_new".to_string())),
                    args: vec![pattern, flags],
                });
            }

            let is_error_class = matches!(
                class_name.as_str(),
                "Error" | "TypeError" | "RangeError" | "ReferenceError" | "SyntaxError" | "URIError"
            );
            if is_error_class {
                let msg_expr = if args.is_empty() {
                    HirExpr::Lit(Literal::String("".to_string()))
                } else {
                    args[0].clone()
                };
                let ctor_fn = if class_name == "Error" {
                    "__bs_Error_new".to_string()
                } else {
                    format!("__bs_{}_new", class_name)
                };
                return Ok(HirExpr::Call {
                    callee: Box::new(HirExpr::GlobalRef(ctor_fn)),
                    args: vec![msg_expr],
                });
            }
            
            Ok(HirExpr::New {
                class_name,
                args,
            })
        } else {
            Err(CompileError::Lowering {
                message: "New expression with non-identifier callee not supported".into(),
            })
        }
    }
}
