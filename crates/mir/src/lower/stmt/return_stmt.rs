use diagnostics::CompileResult;
use hir::HirExpr;
use crate::types::*;
use super::super::LowerCtx;


impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_return(&mut self, val: &Option<HirExpr>) -> CompileResult<()> {
        let v = match val {
            Some(e) => Some(self.lower_expr(e)?),
            None => None,
        };
        self.emit(MirInstr::Return(v));
        Ok(())
    }
}
