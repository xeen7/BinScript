use swc_core::ecma::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_stmt_while(&mut self, w: &WhileStmt, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        let cond = self.lower_expr(&w.test)?;
        let body = self.lower_stmt_to_vec(&w.body)?;
        out.push(HirStmt::While { cond, body });
        Ok(())
    }
}
