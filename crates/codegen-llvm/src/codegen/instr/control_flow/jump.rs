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
    pub(in crate::codegen::instr) fn emit_instr_jump(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::Jump(target) => {
                let current_block = self.builder.get_insert_block().unwrap();
    let current_block_name_cstr = current_block.get_name();
    let current_block_name = current_block_name_cstr.to_str().unwrap();
    let bb = self.bbs[target];
    self.flush_deferred_clears();
    self.builder.build_unconditional_branch(bb).unwrap();
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
