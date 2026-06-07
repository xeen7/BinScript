use oxc::ast::ast::*;

use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_stmt_for_in(&mut self, f: &ForInStatement, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        // Evaluate the object
        let obj_expr = self.lower_expr(&f.right)?;
        
        // Create an array of keys using Object.keys(obj)
        let keys_arr = HirExpr::MemberCall {
            object: "Object".to_string(),
            method: "keys".to_string(),
            args: vec![obj_expr],
        };
        
        let keys_binding = self.declare("_keys");
        out.push(HirStmt::Let {
            binding: keys_binding,
            name: "_keys".to_string(),
            init: Some(keys_arr),
        });
        
        let len_expr = HirExpr::MemberGet {
            object: Box::new(HirExpr::Var(keys_binding)),
            property: "length".to_string(),
        };
        let len_binding = self.declare("_len");
        out.push(HirStmt::Let {
            binding: len_binding,
            name: "_len".to_string(),
            init: Some(len_expr),
        });
        
        let i_binding = self.declare("_i");
        out.push(HirStmt::Let {
            binding: i_binding,
            name: "_i".to_string(),
            init: Some(HirExpr::Lit(Literal::Number(0.0))),
        });
        
        // Loop condition: _i < _len
        let cond = HirExpr::BinOp(
            BinOp::Lt,
            Box::new(HirExpr::Var(i_binding)),
            Box::new(HirExpr::Var(len_binding)),
        );
        
        // Setup loop body
        let mut loop_body = Vec::new();
        
        // Get the current key: _keys[_i]
        let current_key = HirExpr::IndexGet {
            object: Box::new(HirExpr::Var(keys_binding)),
            index: Box::new(HirExpr::Var(i_binding)),
        };
        
        self.push_scope(); // for loop iteration scope
        
        // Bind the current key to the loop variable
        match &f.left {
            ForStatementLeft::VariableDeclaration(var_decl) => {
                for d in &var_decl.declarations {
                    if let BindingPattern::BindingIdentifier(ident) = &d.id {
                        let name = ident.name.to_string();
                        let binding = self.declare(&name);
                        loop_body.push(HirStmt::Let {
                            binding,
                            name,
                            init: Some(current_key.clone()),
                        });
                    }
                }
            }
            left if left.as_assignment_target().is_some() => {
                let target = left.as_assignment_target().unwrap();
                if let AssignmentTarget::AssignmentTargetIdentifier(ident) = target {
                    let name = ident.name.to_string();
                    let binding = self.lookup(&name).ok_or_else(|| CompileError::Lowering {
                        message: format!("Undefined variable in for-in: {}", name),
                    })?;
                    self.reassigned_bindings.insert(binding);
                    loop_body.push(HirStmt::Assign {
                        target: binding,
                        value: current_key.clone(),
                    });
                } else {
                    return Err(CompileError::Lowering {
                        message: "Complex patterns in for-in not yet supported".into()
                    })
                }
            }
            _ => return Err(CompileError::Lowering {
                message: "Unsupported for-in left pattern".into(),
            }),
        }
        
        // Lower user loop body
        match &f.body {
            Statement::BlockStatement(b) => {
                let stmts = self.lower_block_stmts(b)?;
                loop_body.extend(stmts);
            }
            _ => {
                self.lower_stmt(&f.body, &mut loop_body)?;
            }
        }
        self.pop_scope();
        
        // Update _i = _i + 1
        let i_update = HirExpr::Assign {
            target: i_binding,
            value: Box::new(HirExpr::BinOp(
                BinOp::Add,
                Box::new(HirExpr::Var(i_binding)),
                Box::new(HirExpr::Lit(Literal::Number(1.0))),
            )),
        };
        
        out.push(HirStmt::For {
            init: None,
            cond: Some(cond),
            update: Some(i_update),
            body: loop_body,
        });
        
        Ok(())
    }
}
