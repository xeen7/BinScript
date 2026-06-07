use oxc::ast::ast::*;

use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_object(&mut self, obj: &ObjectExpression) -> CompileResult<HirExpr> {
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
        
        for prop_or_spread in &obj.properties {
            match prop_or_spread {
                ObjectPropertyKind::ObjectProperty(p) => {
                    let val_expr = if p.shorthand {
                        if let PropertyKey::StaticIdentifier(id) = &p.key {
                            // Shorthand: value is implicitly an identifier reference
                            match self.lookup(&id.name.to_string()) {
                                Some(bid) => {
                                    self.record_lookup(bid);
                                    HirExpr::Var(bid)
                                }
                                None => HirExpr::GlobalRef(id.name.to_string()),
                            }
                        } else {
                            self.lower_expr(&p.value)?
                        }
                    } else {
                        self.lower_expr(&p.value)?
                    };
                    
                    if p.computed {
                        // For computed properties, the key is an expression
                        // We'll extract it using our helper, but for now we'll match on Expression
                        let key_expr = self.lower_expr(p.key.as_expression().unwrap())?;
                        seq.push(HirExpr::IndexSet {
                            object: Box::new(HirExpr::Var(temp_bid)),
                            index: Box::new(key_expr),
                            value: Box::new(val_expr),
                        });
                    } else {
                        let key_name = match &p.key {
                            PropertyKey::StaticIdentifier(id) => id.name.to_string(),
                            PropertyKey::StringLiteral(s) => s.value.to_string(),
                            PropertyKey::NumericLiteral(n) => n.value.to_string(),
                            _ => return Err(CompileError::Lowering {
                                message: "Unsupported static key type".into(),
                            }),
                        };
                        seq.push(HirExpr::MemberSet {
                            object: Box::new(HirExpr::Var(temp_bid)),
                            property: key_name,
                            value: Box::new(val_expr),
                        });
                    }
                }
                ObjectPropertyKind::SpreadProperty(spread) => {
                    let spread_expr = self.lower_expr(&spread.argument)?;
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
