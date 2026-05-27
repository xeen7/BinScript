use swc_core::ecma::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_stmt_continue(&mut self, c: &ContinueStmt, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        let label = c.label.as_ref().map(|id| id.sym.to_string());
        out.push(HirStmt::Continue(label));
        Ok(())
    }
}
