use swc_core::ecma::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_stmt_break(&mut self, b: &BreakStmt, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        let label = b.label.as_ref().map(|id| id.sym.to_string());
        out.push(HirStmt::Break(label));
        Ok(())
    }
}
