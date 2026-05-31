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
    pub(in crate::codegen::instr) fn emit_instr_try_exit(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::TryExit => {
                let try_exit_fn = self.module.get_function("__bs_try_exit").unwrap();
    self.builder.build_call(try_exit_fn, &[], "").unwrap();
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
