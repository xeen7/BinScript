use diagnostics::CompileResult;
use hir::HirExpr;
use crate::types::*;
use crate::lower::LowerCtx;

mod array;
mod string;
mod date;
mod object;
mod iterator;
mod number;
mod dynamic;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_method_call(
        &mut self,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
    ) -> CompileResult<MirOperand> {
        let obj_operand = self.lower_expr(object)?;

        if let Some(res) = self.lower_method_array(obj_operand.clone(), method, args)? {
            return Ok(res);
        }
        if let Some(res) = self.lower_method_string(obj_operand.clone(), method, args)? {
            return Ok(res);
        }
        if let Some(res) = self.lower_method_date(obj_operand.clone(), method, args)? {
            return Ok(res);
        }
        if let Some(res) = self.lower_method_object(obj_operand.clone(), method, args)? {
            return Ok(res);
        }
        if let Some(res) = self.lower_method_iterator(obj_operand.clone(), method, args)? {
            return Ok(res);
        }
        if let Some(res) = self.lower_method_number(obj_operand.clone(), method, args)? {
            return Ok(res);
        }

        self.lower_method_dynamic(obj_operand, method, args)
    }
}
