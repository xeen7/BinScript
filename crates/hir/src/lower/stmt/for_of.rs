use oxc::ast::ast::*;

use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_stmt_for_of(&mut self, f: &ForOfStatement, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        let left = match &f.left {
            ForStatementLeft::VariableDeclaration(vd) => {
                let mut inits = Vec::new();
                if let BindingPattern::BindingIdentifier(ident) = &vd.declarations[0].id {
                    let name = ident.name.to_string();
                    let binding = self.declare(&name);
                    // The init expression will be synthesized in MIR lowering.
                    // We just create the let binding with no init here.
                    inits.push(HirStmt::Let { binding, name, init: None });
                }
                if inits.len() == 1 {
                    Box::new(inits.into_iter().next().unwrap())
                } else {
                    Box::new(HirStmt::Block(inits))
                }
            }
            left if left.as_assignment_target().is_some() => {
                let target = left.as_assignment_target().unwrap();
                // If it's `for (x of iter)`
                if let AssignmentTarget::AssignmentTargetIdentifier(ident) = target {
                    let name = ident.name.to_string();
                    let binding = self.lookup(&name).ok_or_else(|| CompileError::Lowering {
                        message: format!("Unknown variable {} in for..of", name),
                    })?;
                    // We represent an assignment with a dummy init, which MIR will overwrite.
                    Box::new(HirStmt::Expr(HirExpr::Assign {
                        target: binding,
                        value: Box::new(HirExpr::Lit(Literal::Undefined)),
                    }))
                } else {
                    return Err(CompileError::Lowering {
                        message: "Unsupported for..of left pattern".into(),
                    });
                }
            }
            _ => return Err(CompileError::Lowering {
                message: "Unsupported for..of left pattern".into(),
            }),
        };

        let right = self.lower_expr(&f.right)?;
        let body = self.lower_stmt_to_vec(&f.body)?;
        
        out.push(HirStmt::ForOf { left, right, body, is_await: f.r#await });
        Ok(())
    }
}
