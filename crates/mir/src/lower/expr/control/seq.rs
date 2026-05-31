use diagnostics::CompileResult;
use hir::HirExpr;
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_seq(&mut self, exprs: &[HirExpr]) -> CompileResult<MirOperand> {
        let mut last = MirOperand::ConstUndefined;
        for e in exprs {
            last = self.lower_expr(e)?;
        }
        Ok(last)
    }
}
