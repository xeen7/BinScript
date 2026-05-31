use diagnostics::CompileResult;
use hir::Literal;
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_lit(&mut self, lit: &Literal) -> CompileResult<MirOperand> {
        Ok(match lit {
            Literal::Number(n) => MirOperand::ConstNum(*n),
            Literal::String(s) => MirOperand::ConstStr(s.clone()),
            Literal::Bool(b) => MirOperand::ConstBool(*b),
            Literal::Null => MirOperand::ConstNull,
            Literal::Undefined => MirOperand::ConstUndefined,
        })
    }
}
