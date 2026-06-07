use oxc::ast::ast::*;

use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_lit(&self, expr: &Expression) -> CompileResult<HirExpr> {
        match expr {
            Expression::NumericLiteral(n) => Ok(HirExpr::Lit(Literal::Number(n.value))),
            Expression::StringLiteral(s) => Ok(HirExpr::Lit(Literal::String(
                s.value.to_string(),
            ))),
            Expression::BooleanLiteral(b) => Ok(HirExpr::Lit(Literal::Bool(b.value))),
            Expression::NullLiteral(_) => Ok(HirExpr::Lit(Literal::Null)),
            Expression::RegExpLiteral(r) => {
                Ok(HirExpr::Call {
                    callee: Box::new(HirExpr::GlobalRef("__bs_RegExp_new".to_string())),
                    args: vec![
                        HirExpr::Lit(Literal::String(r.regex.pattern.text.to_string())),
                        HirExpr::Lit(Literal::String(r.regex.flags.to_string())),
                    ],
                })
            }
            Expression::BigIntLiteral(b) => {
                let parsed = b.value.to_string().parse::<f64>().unwrap_or(0.0);
                Ok(HirExpr::Lit(Literal::Number(parsed)))
            }
            _ => Err(CompileError::Lowering {
                message: "Unsupported literal type in Stage 1".into(),
            }),
        }
    }
}
