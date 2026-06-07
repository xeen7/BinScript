use oxc::ast::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_stmt_continue(&mut self, c: &ContinueStatement, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        let label = c.label.as_ref().map(|id| id.name.to_string());
        out.push(HirStmt::Continue(label));
        Ok(())
    }
}
