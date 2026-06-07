use oxc::ast::ast::*;

use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::{LowerCtx, conv_bin_op, conv_logical_op};

impl LowerCtx {
    pub(super) fn lower_expr_bin(&mut self, bin: &BinaryExpression) -> CompileResult<HirExpr> {
        let op = conv_bin_op(bin.operator);
        if bin.operator == BinaryOperator::Instanceof {
            if let Expression::Identifier(right_id) = &bin.right {
                let left_expr = self.lower_expr(&bin.left)?;
                let raw_class_name = right_id.name.to_string();
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

    pub(super) fn lower_expr_logical(&mut self, logical: &LogicalExpression) -> CompileResult<HirExpr> {
        let op = conv_logical_op(logical.operator);
        let l = self.lower_expr(&logical.left)?;
        let r = self.lower_expr(&logical.right)?;
        Ok(HirExpr::BinOp(op, Box::new(l), Box::new(r)))
    }
}
