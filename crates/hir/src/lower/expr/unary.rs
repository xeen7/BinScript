use swc_core::ecma::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::{LowerCtx, conv_unary_op};

impl LowerCtx {
    pub(super) fn lower_expr_unary(&mut self, u: &UnaryExpr) -> CompileResult<HirExpr> {
        if u.op == swc_core::ecma::ast::UnaryOp::Delete {
            if let Expr::Member(m) = &*u.arg {
                let obj = self.lower_expr(&m.obj)?;
                let prop = match &m.prop {
                    MemberProp::Ident(id) => HirExpr::Lit(Literal::String(id.sym.to_string())),
                    MemberProp::Computed(comp) => self.lower_expr(&comp.expr)?,
                    MemberProp::PrivateName(_) => return Err(diagnostics::CompileError::Lowering {
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
            let _ = self.lower_expr(&u.arg)?;
            return Ok(HirExpr::Lit(Literal::Bool(true)));
        }

        let op = conv_unary_op(u.op);
        let arg = self.lower_expr(&u.arg)?;
        Ok(HirExpr::UnaryOp(op, Box::new(arg)))
    }
}
