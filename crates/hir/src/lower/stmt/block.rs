use oxc::ast::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_stmt_block(&mut self, b: &BlockStatement, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        self.push_scope();
        let stmts = self.lower_block_stmts(b)?;
        self.pop_scope();
        out.push(HirStmt::Block(stmts));
        Ok(())
    }
}
