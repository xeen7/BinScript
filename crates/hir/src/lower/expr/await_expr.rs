use oxc::ast::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_await(&mut self, a: &AwaitExpression) -> CompileResult<HirExpr> {
        let inner = self.lower_expr(&a.argument)?;
        Ok(HirExpr::Await(Box::new(inner)))
    }
}
