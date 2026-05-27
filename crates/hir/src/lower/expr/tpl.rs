use swc_core::ecma::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_tpl(&mut self, tpl: &Tpl) -> CompileResult<HirExpr> {
        self.lower_template(tpl)
    }

    fn lower_template(&mut self, tpl: &Tpl) -> CompileResult<HirExpr> {
        let mut parts: Vec<HirExpr> = Vec::new();
        for (i, quasi) in tpl.quasis.iter().enumerate() {
            let raw = quasi.raw.to_string();
            if !raw.is_empty() {
                parts.push(HirExpr::Lit(Literal::String(raw)));
            }
            if i < tpl.exprs.len() {
                parts.push(self.lower_expr(&tpl.exprs[i])?);
            }
        }
        if parts.is_empty() {
            return Ok(HirExpr::Lit(Literal::String(String::new())));
        }
        if parts.len() == 1 {
            return Ok(parts.into_iter().next().unwrap());
        }
        // Chain binary Add for string concatenation
        let mut result = parts.remove(0);
        for part in parts {
            result = HirExpr::BinOp(BinOp::Add, Box::new(result), Box::new(part));
        }
        Ok(result)
    }

    pub(super) fn lower_expr_tagged_tpl(&mut self, tt: &TaggedTpl) -> CompileResult<HirExpr> {
        let mut strings_exprs = Vec::new();
        for quasi in &tt.tpl.quasis {
            strings_exprs.push(HirExpr::Lit(Literal::String(quasi.raw.to_string())));
        }
        let strings_array = HirExpr::ArrayLit(strings_exprs);

        let mut call_args = vec![strings_array];
        for expr in &tt.tpl.exprs {
            call_args.push(self.lower_expr(expr)?);
        }

        let callee = self.lower_expr(&tt.tag)?;
        Ok(HirExpr::Call {
            callee: Box::new(callee),
            args: call_args,
        })
    }
}
