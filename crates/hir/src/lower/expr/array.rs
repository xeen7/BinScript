use swc_core::ecma::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_array(&mut self, arr: &ArrayLit) -> CompileResult<HirExpr> {
        let mut elems = Vec::new();
        for elem in &arr.elems {
            match elem {
                Some(expr_or_spread) => {
                    if expr_or_spread.spread.is_some() {
                        let inner = self.lower_expr(&expr_or_spread.expr)?;
                        elems.push(HirExpr::Spread(Box::new(inner)));
                    } else {
                        elems.push(self.lower_expr(&expr_or_spread.expr)?);
                    }
                }
                None => {
                    elems.push(HirExpr::Lit(Literal::Undefined));
                }
            }
        }
        Ok(HirExpr::ArrayLit(elems))
    }
}
