use swc_core::ecma::ast::*;

use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::{LowerCtx, conv_bin_op};

impl LowerCtx {
    pub(super) fn lower_expr_bin(&mut self, bin: &BinExpr) -> CompileResult<HirExpr> {
        let op = conv_bin_op(bin.op);
        if bin.op == BinaryOp::InstanceOf {
            if let Expr::Ident(right_id) = &*bin.right {
                let left_expr = self.lower_expr(&bin.left)?;
                let raw_class_name = right_id.sym.to_string();
                let class_name = self.class_aliases.get(&raw_class_name)
                    .cloned()
                    .unwrap_or(raw_class_name);
                return Ok(HirExpr::InstanceOf {
                    expr: Box::new(left_expr),
                    class_name,
                });
            } else {
                return Err(CompileError::Lowering {
                    message: "instanceof RHS must be a class name".into(),
                });
            }
        }
        let l = self.lower_expr(&bin.left)?;
        let r = self.lower_expr(&bin.right)?;
        Ok(HirExpr::BinOp(op, Box::new(l), Box::new(r)))
    }
}
