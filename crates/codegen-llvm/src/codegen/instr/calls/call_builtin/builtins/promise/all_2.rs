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
    pub(in crate::codegen::instr) fn emit_builtin_promise_all_2(&mut self, d: &MirReg, args: &[MirOperand]) -> CompileResult<()> {
let f = self.module.get_function("__bs_promise_all_2").unwrap();
    let a1 = self.val(&args[0])?;
    let a2 = self.val(&args[1])?;
    let rv = self.builder.build_call(f, &[a1.into(), a2.into()], "promise_all_2").unwrap();
    let v = rv.try_as_basic_value().basic().unwrap().into_int_value();
    self.store(*d, v);
        Ok(())
    }
}
