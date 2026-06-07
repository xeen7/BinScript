use oxc::ast::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_cond(&mut self, c: &ConditionalExpression) -> CompileResult<HirExpr> {
        let cond = self.lower_expr(&c.test)?;
        let then_e = self.lower_expr(&c.consequent)?;
        let else_e = self.lower_expr(&c.alternate)?;
        Ok(HirExpr::Ternary {
            cond: Box::new(cond),
            then_expr: Box::new(then_e),
            else_expr: Box::new(else_e),
        })
    }
}
