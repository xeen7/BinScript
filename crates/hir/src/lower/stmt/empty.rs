use oxc::ast::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_stmt_empty(&mut self, _e: &EmptyStatement, _out: &mut Vec<HirStmt>) -> CompileResult<()> {
        Ok(())
    }
}
