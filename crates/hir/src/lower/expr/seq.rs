use oxc::ast::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_seq(&mut self, seq: &SequenceExpression) -> CompileResult<HirExpr> {
        let exprs = seq.expressions.iter()
            .map(|e| self.lower_expr(e))
            .collect::<CompileResult<Vec<_>>>()?;
        Ok(HirExpr::Seq(exprs))
    }
}
