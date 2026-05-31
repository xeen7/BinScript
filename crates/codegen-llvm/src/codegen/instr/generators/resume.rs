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
    pub(in crate::codegen::instr) fn emit_instr_resume(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::Resume(dest, idx) => {
                let resume_bb = self.resume_blocks[idx];
    self.builder.position_at_end(resume_bb);
    
    let state_ptr = self.gen_state_ptr.unwrap();
    let state_ty = self.gen_state_ty.unwrap();
    
    for (rid, alloca) in &self.regs {
        let slot = unsafe { self.builder.build_struct_gep(state_ty, state_ptr, 2 + self.gen_num_args + *rid, "slot").unwrap() };
        let loaded = self.builder.build_load(self.i64_ty, slot, "restored").unwrap().into_int_value();
        self.builder.build_store(*alloca, loaded).unwrap();
    }
    
    let sent_val = self.gen_sent_val.unwrap();
    self.store(*dest, sent_val);
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
