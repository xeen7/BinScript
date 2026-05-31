use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_spread(&self) -> CompileResult<MirOperand> {
        Err(CompileError::Lowering {
            message: "Spread expression outside array/object literals is unsupported".into(),
        })
    }
}
