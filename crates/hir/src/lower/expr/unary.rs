use oxc::ast::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::{LowerCtx, conv_unary_op};

impl LowerCtx {
    pub(super) fn lower_expr_unary(&mut self, u: &UnaryExpression) -> CompileResult<HirExpr> {
        if u.operator == UnaryOperator::Delete {
            if let Some(m) = u.argument.as_member_expression() {
                let obj = self.lower_expr(m.object())?;
                let prop = match m {
                    MemberExpression::StaticMemberExpression(s) => HirExpr::Lit(Literal::String(s.property.name.to_string())),
                    MemberExpression::ComputedMemberExpression(c) => self.lower_expr(&c.expression)?,
                    MemberExpression::PrivateFieldExpression(_) => return Err(diagnostics::CompileError::Lowering {
                        message: "Cannot delete private properties".to_string(),
                    }),
                };
                return Ok(HirExpr::DeleteProp {
                    object: Box::new(obj),
                    property: Box::new(prop),
                });
            }
            // `delete foo` on a plain identifier or value in strict mode is either an error or returns true, 
            // but we'll fall back to evaluating it and returning true to be safe, 
            // though standard JS delete on non-members returns true.
            let _ = self.lower_expr(&u.argument)?;
            return Ok(HirExpr::Lit(Literal::Bool(true)));
        }

        let op = conv_unary_op(u.operator);
        let arg = self.lower_expr(&u.argument)?;
        Ok(HirExpr::UnaryOp(op, Box::new(arg)))
    }
}
