use swc_core::ecma::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_await(&mut self, a: &AwaitExpr) -> CompileResult<HirExpr> {
        let inner = self.lower_expr(&a.arg)?;
        Ok(HirExpr::Await(Box::new(inner)))
    }
}
