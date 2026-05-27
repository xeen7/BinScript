use swc_core::ecma::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_stmt_for(&mut self, f: &ForStmt, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        self.lower_for(f, out)
    }

    fn lower_for(&mut self, f: &ForStmt, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        let init = match &f.init {
            Some(VarDeclOrExpr::VarDecl(vd)) => {
                let mut inits = Vec::new();
                for d in &vd.decls {
                    if let Pat::Ident(ident) = &d.name {
                        let name = ident.sym.to_string();
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
            Some(VarDeclOrExpr::Expr(e)) => {
                let expr = self.lower_expr(e)?;
                Some(Box::new(HirStmt::Expr(expr)))
            }
            None => None,
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
