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
    self.flush_deferred_clears();
    self.emit_call_with_invoke(throw_fn, &[val.into()], "throw_call").unwrap();
    
    // __bs_throw is noreturn, but LLVM needs a terminator for the basic block to be valid.
    // We emit a dummy return instead of unreachable! to ensure the call instruction is not the very last byte
    // of the function, which could confuse the unwinder's Return Address lookup.
    let curr_fn = self.builder.get_insert_block().unwrap().get_parent().unwrap();
    if let Some(ret_type) = curr_fn.get_type().get_return_type() {
        if ret_type.is_int_type() && ret_type.into_int_type().get_bit_width() == 32 {
            self.builder.build_return(Some(&self.i32_ty.const_int(0, false))).unwrap();
        } else {
            self.builder.build_return(Some(&self.nan.const_undefined())).unwrap();
        }
    } else {
        self.builder.build_return(None).unwrap();
    }
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
