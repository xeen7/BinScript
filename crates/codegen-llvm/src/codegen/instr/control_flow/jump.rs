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
    
    // If target block ID is less than current block ID (or we can't easily tell so we just check the target name vs current),
    // it's a back-edge. A simple heuristic for now is to just look if the target id is <= current block's parsed id.
    let mut is_back_edge = false;
    if let Some(curr_id_str) = current_block_name.strip_prefix("bb") {
        if let Ok(curr_id) = curr_id_str.parse::<u32>() {
            if *target <= curr_id {
                is_back_edge = true;
            }
        }
    }
    
    if is_back_edge {
        let safepoint_poll = self.module.get_function("__bs_safepoint_poll").unwrap();
        self.builder.build_call(safepoint_poll, &[], "safepoint_poll").unwrap();
    }

    self.builder.build_unconditional_branch(bb).unwrap();
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
