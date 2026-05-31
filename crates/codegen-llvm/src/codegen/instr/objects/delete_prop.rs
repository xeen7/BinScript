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
    pub(in crate::codegen::instr) fn emit_instr_delete_prop(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::DeleteProp(dest, obj, prop) => {
                let ov = self.val(obj)?;
    let pv = self.val(prop)?;
    let func = self.module.get_function("__bs_delete_prop").unwrap();
    let call = self.builder.build_call(func, &[ov.into(), pv.into()], "del_prop_call").unwrap();
    let res = call.try_as_basic_value().basic().unwrap().into_int_value();
    self.store(*dest, res);
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
