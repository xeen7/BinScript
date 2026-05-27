use swc_core::ecma::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_paren(&mut self, p: &ParenExpr) -> CompileResult<HirExpr> {
        self.lower_expr(&p.expr)
    }
}
