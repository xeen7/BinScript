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
    pub(in crate::codegen::instr) fn emit_builtin_generator_next(&mut self, d: &MirReg, args: &[MirOperand]) -> CompileResult<()> {
let gen_next = self.module.get_function("__bs_generator_next").unwrap();
    let gen_ptr = self.val(&args[0])?;
    let sent_val = self.val(&args[1])?;
    let rv = self.builder.build_call(gen_next, &[gen_ptr.into(), sent_val.into()], "gen_next").unwrap();
    let v = rv.try_as_basic_value().basic().unwrap().into_int_value();
    self.store(*d, v);
        Ok(())
    }
}
