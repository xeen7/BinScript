use swc_core::ecma::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_stmt_expr(&mut self, es: &ExprStmt, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        let expr = self.lower_expr(&es.expr)?;
        out.push(HirStmt::Expr(expr));
        Ok(())
    }
}
