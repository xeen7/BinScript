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
    pub(in crate::codegen::instr) fn emit_instr_throw(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::Throw(val_operand) => {
                let val = self.val(val_operand)?;
    let throw_fn = self.module.get_function("__bs_throw").unwrap();
    self.builder.build_call(throw_fn, &[val.into()], "").unwrap();
    
    // __bs_throw is noreturn, but LLVM needs a terminator for the basic block to be valid.
    self.builder.build_unreachable().unwrap();
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
