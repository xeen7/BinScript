use swc_core::ecma::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_seq(&mut self, seq: &SeqExpr) -> CompileResult<HirExpr> {
        let exprs = seq.exprs.iter()
            .map(|e| self.lower_expr(e))
            .collect::<CompileResult<Vec<_>>>()?;
        Ok(HirExpr::Seq(exprs))
    }
}
