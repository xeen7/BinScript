use oxc::ast::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_yield(&mut self, y: &YieldExpression) -> CompileResult<HirExpr> {
        let inner = match &y.argument {
            Some(expr) => Some(Box::new(self.lower_expr(expr)?)),
            None => None,
        };
        Ok(HirExpr::Yield {
            arg: inner,
            delegate: y.delegate,
        })
    }
}
