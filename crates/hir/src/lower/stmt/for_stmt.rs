use oxc::ast::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_stmt_for(&mut self, f: &ForStatement, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        self.lower_for(f, out)
    }

    fn lower_for(&mut self, f: &ForStatement, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        let init = match &f.init {
            Some(ForStatementInit::VariableDeclaration(vd)) => {
                let mut inits = Vec::new();
                for d in &vd.declarations {
                    if let BindingPattern::BindingIdentifier(ident) = &d.id {
                        let name = ident.name.to_string();
                        let binding = self.declare(&name);
                        let init_expr = match &d.init {
                            Some(e) => Some(self.lower_expr(e)?),
                            None => None,
                        };
                        inits.push(HirStmt::Let { binding, name, init: init_expr });
                    }
                }
                if inits.len() == 1 {
                    Some(Box::new(inits.into_iter().next().unwrap()))
                } else {
                    Some(Box::new(HirStmt::Block(inits)))
                }
            }
            Some(init) if init.as_expression().is_some() => {
                let e = init.as_expression().unwrap();
                let expr = self.lower_expr(e)?;
                Some(Box::new(HirStmt::Expr(expr)))
            }
            None => None,
            _ => return Err(diagnostics::CompileError::Lowering {
                message: "Unsupported for-loop init".into(),
            }),
        };
        let cond = match &f.test {
            Some(e) => Some(self.lower_expr(e)?),
            None => None,
        };
        let update = match &f.update {
            Some(e) => Some(self.lower_expr(e)?),
            None => None,
        };
        let body = self.lower_stmt_to_vec(&f.body)?;
        out.push(HirStmt::For { init, cond, update, body });
        Ok(())
    }
}
