use oxc::ast::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_stmt_do_while(&mut self, dw: &DoWhileStatement, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        let body = self.lower_stmt_to_vec(&dw.body)?;
        let cond = self.lower_expr(&dw.test)?;
        out.push(HirStmt::DoWhile { body, cond });
        Ok(())
    }
}
