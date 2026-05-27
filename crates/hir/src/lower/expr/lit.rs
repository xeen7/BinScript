use swc_core::ecma::ast::*;

use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_lit(&self, lit: &Lit) -> CompileResult<HirExpr> {
        match lit {
            Lit::Num(n) => Ok(HirExpr::Lit(Literal::Number(n.value))),
            Lit::Str(s) => Ok(HirExpr::Lit(Literal::String(
                s.value.as_wtf8().to_string_lossy().into_owned(),
            ))),
            Lit::Bool(b) => Ok(HirExpr::Lit(Literal::Bool(b.value))),
            Lit::Null(_) => Ok(HirExpr::Lit(Literal::Null)),
            Lit::Regex(r) => {
                Ok(HirExpr::Call {
                    callee: Box::new(HirExpr::GlobalRef("__bs_RegExp_new".to_string())),
                    args: vec![
                        HirExpr::Lit(Literal::String(r.exp.to_string())),
                        HirExpr::Lit(Literal::String(r.flags.to_string())),
                    ],
                })
            }
            Lit::BigInt(b) => {
                let parsed = format!("{}", b.value).parse::<f64>().unwrap_or(0.0);
                Ok(HirExpr::Lit(Literal::Number(parsed)))
            }
            _ => Err(CompileError::Lowering {
                message: "Unsupported literal type in Stage 1".into(),
            }),
        }
    }
}
