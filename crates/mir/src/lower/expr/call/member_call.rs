use diagnostics::{CompileError, CompileResult};
use hir::HirExpr;
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_member_call(
        &mut self,
        object: &str,
        method: &str,
        args: &[HirExpr],
    ) -> CompileResult<MirOperand> {
        let mir_args: Vec<MirOperand> = args
            .iter()
            .map(|a| self.lower_expr(a))
            .collect::<CompileResult<_>>()?;
        let dest = self.fresh_reg();
        if self.lower_builtin_member_call(object, method, mir_args, dest)? {
            return Ok(MirOperand::Reg(dest));
        }
        Err(CompileError::Lowering {
            message: format!("{}.{}() not supported", object, method),
        })
    }
}
