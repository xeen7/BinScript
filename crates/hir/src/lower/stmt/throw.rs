use oxc::ast::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_stmt_throw(&mut self, t: &ThrowStatement, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        let arg = self.lower_expr(&t.argument)?;
        out.push(HirStmt::Throw(arg));
        Ok(())
    }
}
