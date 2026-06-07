use oxc::ast::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

enum ChainLink<'a> {
    Member { property: String, optional: bool },
    Index { index_expr: &'a Expression<'a>, optional: bool },
    Call { args: &'a oxc::allocator::Vec<'a, Argument<'a>>, optional: bool },
}

impl<'a> ChainLink<'a> {
    fn is_optional(&self) -> bool {
        match self {
            ChainLink::Member { optional, .. } => *optional,
            ChainLink::Index { optional, .. } => *optional,
            ChainLink::Call { optional, .. } => *optional,
        }
    }
}

impl LowerCtx {
    pub(super) fn lower_expr_opt_chain(&mut self, e: &ChainExpression) -> CompileResult<HirExpr> {
        let mut links = Vec::new();
        let base = self.unwrap_opt_chain(e, &mut links)?;
        
        let base_hir = self.lower_expr(base)?;
        
        let temp_bid = self.next_binding;
        self.next_binding += 1;
        let current_func = *self.function_stack.last().unwrap_or(&0);
        self.binding_owners.insert(temp_bid, current_func);
        
        let init_assign = HirExpr::Assign {
            target: temp_bid,
            value: Box::new(base_hir),
        };
        
        let rest_expr = self.build_chain_expr(temp_bid, &links)?;
        
        Ok(HirExpr::Seq(vec![init_assign, rest_expr]))
    }
    
    fn unwrap_opt_chain<'a>(&self, e: &'a ChainExpression, links: &mut Vec<ChainLink<'a>>) -> CompileResult<&'a Expression<'a>> {
        self.unwrap_opt_chain_element(&e.expression, links)
    }
    
    fn unwrap_opt_chain_element<'a>(&self, expr: &'a ChainElement<'a>, links: &mut Vec<ChainLink<'a>>) -> CompileResult<&'a Expression<'a>> {
        match expr {
            ChainElement::CallExpression(c) => {
                let optional = c.optional;
                let base = self.unwrap_opt_chain_expr(&c.callee, links)?;
                links.push(ChainLink::Call {
                    args: &c.arguments,
                    optional,
                });
                Ok(base)
            }
            m => {
                // MemberExpression variants are flattened into ChainElement
                let me = m.as_member_expression().ok_or_else(|| {
                    diagnostics::CompileError::Lowering {
                        message: "Unsupported chain element type".into(),
                    }
                })?;
                let optional = false; // TODO: extract optional from chain element
                let base = self.unwrap_opt_chain_expr(me.object(), links)?;
                match me {
                    MemberExpression::StaticMemberExpression(sme) => {
                        links.push(ChainLink::Member {
                            property: sme.property.name.to_string(),
                            optional,
                        });
                    }
                    MemberExpression::ComputedMemberExpression(cme) => {
                        links.push(ChainLink::Index {
                            index_expr: &cme.expression,
                            optional,
                        });
                    }
                    MemberExpression::PrivateFieldExpression(pfe) => {
                        links.push(ChainLink::Member {
                            property: format!("__private_{}", pfe.field.name),
                            optional,
                        });
                    }
                }
                Ok(base)
            }
        }
    }
    
    fn unwrap_opt_chain_expr<'a>(&self, expr: &'a Expression<'a>, links: &mut Vec<ChainLink<'a>>) -> CompileResult<&'a Expression<'a>> {
        match expr {
            Expression::ChainExpression(o) => {
                self.unwrap_opt_chain(o, links)
            }
            _ => Ok(expr)
        }
    }
    
    fn build_chain_expr(&mut self, temp_bid: BindingId, links: &[ChainLink]) -> CompileResult<HirExpr> {
        if links.is_empty() {
            return Ok(HirExpr::Var(temp_bid));
        }
        
        let link = &links[0];
        let rest = &links[1..];
        
        let op_expr = match link {
            ChainLink::Member { property, .. } => {
                HirExpr::MemberGet {
                    object: Box::new(HirExpr::Var(temp_bid)),
                    property: property.clone(),
                }
            }
            ChainLink::Index { index_expr, .. } => {
                let idx = self.lower_expr(index_expr)?;
                HirExpr::IndexGet {
                    object: Box::new(HirExpr::Var(temp_bid)),
                    index: Box::new(idx),
                }
            }
            ChainLink::Call { args, .. } => {
                let mut lowered_args = Vec::new();
                for arg in args.iter() {
                    match arg {
                        Argument::SpreadElement(spread) => {
                            let inner = self.lower_expr(&spread.argument)?;
                            lowered_args.push(HirExpr::Spread(Box::new(inner)));
                        }
                        other => {
                            let expr = other.as_expression().unwrap();
                            lowered_args.push(self.lower_expr(expr)?);
                        }
                    }
                }
                HirExpr::Call {
                    callee: Box::new(HirExpr::Var(temp_bid)),
                    args: lowered_args,
                }
            }
        };
        
        let assign_expr = HirExpr::Assign {
            target: temp_bid,
            value: Box::new(op_expr),
        };
        
        if link.is_optional() {
            let is_null = HirExpr::BinOp(
                BinOp::StrictEq,
                Box::new(HirExpr::Var(temp_bid)),
                Box::new(HirExpr::Lit(Literal::Null)),
            );
            let is_undefined = HirExpr::BinOp(
                BinOp::StrictEq,
                Box::new(HirExpr::Var(temp_bid)),
                Box::new(HirExpr::Lit(Literal::Undefined)),
            );
            let cond = HirExpr::BinOp(
                BinOp::Or,
                Box::new(is_null),
                Box::new(is_undefined),
            );
            
            let else_branch = if rest.is_empty() {
                assign_expr
            } else {
                let rest_expr = self.build_chain_expr(temp_bid, rest)?;
                HirExpr::Seq(vec![assign_expr, rest_expr])
            };
            
            Ok(HirExpr::Ternary {
                cond: Box::new(cond),
                then_expr: Box::new(HirExpr::Lit(Literal::Undefined)),
                else_expr: Box::new(else_branch),
            })
        } else {
            if rest.is_empty() {
                Ok(assign_expr)
            } else {
                let rest_expr = self.build_chain_expr(temp_bid, rest)?;
                Ok(HirExpr::Seq(vec![assign_expr, rest_expr]))
            }
        }
    }
}
