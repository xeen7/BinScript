use oxc::ast::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_array(&mut self, arr: &ArrayExpression) -> CompileResult<HirExpr> {
        let mut elems = Vec::new();
        for elem in &arr.elements {
            match elem {
                ArrayExpressionElement::SpreadElement(spread) => {
                    let inner = self.lower_expr(&spread.argument)?;
                    elems.push(HirExpr::Spread(Box::new(inner)));
                }
                elem if elem.as_expression().is_some() => {
                    let expr_inner = elem.as_expression().unwrap();
                    // was: ArrayExpressionElement::Expression(expr) => {
                    elems.push(self.lower_expr(expr_inner)?);
                }
                ArrayExpressionElement::Elision(_) => {
                    elems.push(HirExpr::Lit(Literal::Undefined));
                }
                _ => return Err(diagnostics::CompileError::Lowering {
                    message: "Unsupported array element".into(),
                }),
            }
        }
        Ok(HirExpr::ArrayLit(elems))
    }
}
