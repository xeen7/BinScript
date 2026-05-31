#![allow(unused_imports)]
#![allow(unreachable_code)]
#![allow(unused_unsafe)]
use inkwell::values::BasicMetadataValueEnum;
use inkwell::IntPredicate;
use inkwell::FloatPredicate;
use mir::types::*;
use mir::BuiltinFn;
use diagnostics::{CompileError, CompileResult};

use crate::codegen::LlvmCodegen;

impl<'ctx> LlvmCodegen<'ctx> {
    #[allow(unused_variables)]
    pub(in crate::codegen::instr) fn emit_builtin_array_new(&mut self, d: &MirReg, args: &[MirOperand]) -> CompileResult<()> {
        return Err(CompileError::Codegen {
            message: "BuiltinFn::ArrayNew not yet implemented in codegen".into(),
        });
        Ok(())
    }
}
