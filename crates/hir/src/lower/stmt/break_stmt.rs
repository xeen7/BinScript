use oxc::ast::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_stmt_break(&mut self, b: &BreakStatement, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        let label = b.label.as_ref().map(|id| id.name.to_string());
        out.push(HirStmt::Break(label));
        Ok(())
    }
}
