use swc_core::ecma::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_stmt_return(&mut self, r: &ReturnStmt, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        let value = match &r.arg {
            Some(e) => Some(self.lower_expr(e)?),
            None => None,
        };
        out.push(HirStmt::Return(value));
        Ok(())
    }
}
