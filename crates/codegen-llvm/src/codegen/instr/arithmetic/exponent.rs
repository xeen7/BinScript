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
    #[allow(unreachable_code)]
    #[allow(unused_variables)]
    pub(in crate::codegen::instr) fn emit_instr_exponent(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::Exp(d, l, r) => {
                let lv = self.val(l)?;
    let rv = self.val(r)?;
    let func = self.module.get_function("__bs_exp").unwrap();
    let call = self.builder.build_call(func, &[lv.into(), rv.into()], "exp_call").unwrap();
    let res = call.try_as_basic_value().basic().unwrap().into_int_value();
    self.store(*d, res);
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
