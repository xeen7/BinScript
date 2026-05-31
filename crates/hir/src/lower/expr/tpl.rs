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

        // Bundle interpolated values into a single array argument
        // so that `tag\`...\`` becomes `tag(strings, [val1, val2, ...])`
        // matching the `...values` rest parameter convention.
        let mut values_exprs = Vec::new();
        for expr in &tt.tpl.exprs {
            values_exprs.push(self.lower_expr(expr)?);
        }
        let values_array = HirExpr::ArrayLit(values_exprs);

        let call_args = vec![strings_array, values_array];

        let callee = match &*tt.tag {
            Expr::Ident(id) => {
                let name = id.sym.to_string();
                if self.function_names.contains(&name) {
                    HirExpr::GlobalRef(name)
                } else if let Some(aliased) = self.function_aliases.get(&name) {
                    HirExpr::GlobalRef(aliased.clone())
                } else {
                    self.lookup(&name)
                        .map(HirExpr::Var)
                        .unwrap_or_else(|| HirExpr::GlobalRef(name))
                }
            }
            _ => self.lower_expr(&tt.tag)?,
        };

        Ok(HirExpr::Call {
            callee: Box::new(callee),
            args: call_args,
        })
    }
}
