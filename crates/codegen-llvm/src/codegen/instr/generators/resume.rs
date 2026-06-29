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
                
                let sent_val = self.gen_sent_val.unwrap();
                self.store(*dest, sent_val);
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
