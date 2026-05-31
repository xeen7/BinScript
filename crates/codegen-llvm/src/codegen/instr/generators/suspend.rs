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
    pub(in crate::codegen::instr) fn emit_instr_suspend(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::Suspend(idx, val) => {
                let v = self.val(val)?;
    let state_ptr = self.gen_state_ptr.unwrap();
    let state_ty = self.gen_state_ty.unwrap();
    
    for (rid, alloca) in &self.regs {
        let val_to_save = self.builder.build_load(self.i64_ty, *alloca, "saved").unwrap().into_int_value();
        let slot = unsafe { self.builder.build_struct_gep(state_ty, state_ptr, 2 + self.gen_num_args + *rid, "slot").unwrap() };
        self.builder.build_store(slot, val_to_save).unwrap();
    }
    
    let state_idx_ptr = unsafe { self.builder.build_struct_gep(state_ty, state_ptr, 0, "state_idx_ptr").unwrap() };
    self.builder.build_store(state_idx_ptr, self.i64_ty.const_int((*idx + 1) as u64, false)).unwrap();
    
    self.builder.build_return(Some(&v)).unwrap();
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
