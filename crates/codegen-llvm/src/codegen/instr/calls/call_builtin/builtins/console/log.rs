#![allow(unused_imports)]
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
    pub(in crate::codegen::instr) fn emit_builtin_console_log(&mut self, d: &MirReg, args: &[MirOperand]) -> CompileResult<()> {
let log1 = self.funcs["__bs_console_log_1"];
    for a in args {
        let v = self.val(a)?;
        self.builder.build_call(log1, &[v.into()], "").unwrap();
    }
    self.store(*d, self.nan.const_undefined());
        Ok(())
    }
}
