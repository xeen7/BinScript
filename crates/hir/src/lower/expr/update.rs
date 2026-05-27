use swc_core::ecma::ast::*;

use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_update(&mut self, u: &UpdateExpr) -> CompileResult<HirExpr> {
        // ++i / i++ / --i / i-- → desugar to i = i ± 1
        if let Expr::Ident(id) = &*u.arg {
            let name = id.sym.to_string();
            let binding = self.lookup(&name).ok_or_else(|| CompileError::Lowering {
                message: format!("Undefined variable in update: {}", name),
            })?;
            self.record_lookup(binding);
            self.reassigned_bindings.insert(binding);
            self.const_strings.remove(&binding);
            let op = match u.op {
                UpdateOp::PlusPlus => BinOp::Add,
                UpdateOp::MinusMinus => BinOp::Sub,
            };
            let one = HirExpr::Lit(Literal::Number(1.0));
            let cur = HirExpr::Var(binding);
            let new_val = HirExpr::BinOp(op, Box::new(cur), Box::new(one));
            Ok(HirExpr::Assign { target: binding, value: Box::new(new_val) })
        } else {
            Err(CompileError::Lowering {
                message: "Complex update targets not supported".into(),
            })
        }
    }
}
