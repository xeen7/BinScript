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
    pub(in crate::codegen::instr) fn emit_instr_return_op(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::Return(v) => {
                {
    let rv = match v {
        Some(op) => self.val(op)?,
        None => self.nan.const_undefined(),
    };
    if let Some(state_ptr) = self.gen_state_ptr {
        let state_ty = self.gen_state_ty.unwrap();
        let state_idx_ptr = unsafe { self.builder.build_struct_gep(state_ty, state_ptr, 0, "state_idx_ptr").unwrap() };
        self.builder.build_store(state_idx_ptr, self.i64_ty.const_all_ones()).unwrap();
    }
    self.builder.build_return(Some(&rv)).unwrap();
}

// --- Stage 2 additions ---
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
