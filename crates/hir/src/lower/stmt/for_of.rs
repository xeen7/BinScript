use swc_core::ecma::ast::*;

use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_stmt_for_of(&mut self, f: &ForOfStmt, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        let left = match &f.left {
            ForHead::VarDecl(vd) => {
                let mut inits = Vec::new();
                for d in &vd.decls {
                    if let Pat::Ident(ident) = &d.name {
                        let name = ident.sym.to_string();
                        let binding = self.declare(&name);
                        // The init expression will be synthesized in MIR lowering.
                        // We just create the let binding with no init here.
                        inits.push(HirStmt::Let { binding, name, init: None });
                    }
                }
                if inits.len() == 1 {
                    Box::new(inits.into_iter().next().unwrap())
                } else {
                    Box::new(HirStmt::Block(inits))
                }
            }
            ForHead::Pat(pat) => {
                // If it's `for (x of iter)`, left is Pat::Ident(x)
                if let Pat::Ident(ident) = &**pat {
                    let name = ident.sym.to_string();
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
            _ => {
                return Err(CompileError::Lowering {
                    message: "Unsupported for..of head".into(),
                });
            }
        };

        let right = self.lower_expr(&f.right)?;
        let body = self.lower_stmt_to_vec(&f.body)?;
        
        out.push(HirStmt::ForOf { left, right, body, is_await: f.is_await });
        Ok(())
    }
}
