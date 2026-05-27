use swc_core::ecma::ast::*;

use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_this(&self, _this: &ThisExpr) -> CompileResult<HirExpr> {
        if let Some(this_id) = self.this_binding {
            Ok(HirExpr::Var(this_id))
        } else {
            Err(CompileError::Lowering {
                message: "'this' used outside class method or constructor".into(),
            })
        }
    }
}
