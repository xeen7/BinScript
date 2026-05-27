use swc_core::ecma::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_stmt_if(&mut self, if_s: &IfStmt, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        let cond = self.lower_expr(&if_s.test)?;
        let then_body = self.lower_stmt_to_vec(&if_s.cons)?;
        let else_body = match &if_s.alt {
            Some(s) => Some(self.lower_stmt_to_vec(s)?),
            None => None,
        };
        out.push(HirStmt::If { cond, then_body, else_body });
        Ok(())
    }
}
