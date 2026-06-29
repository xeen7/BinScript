use inkwell::values::BasicMetadataValueEnum;
use mir::types::MirInstr;
use diagnostics::{CompileError, CompileResult};

use crate::codegen::LlvmCodegen;

impl<'ctx> LlvmCodegen<'ctx> {
    pub(in crate::codegen::instr) fn emit_instr_arena_create(&mut self, instr: &MirInstr) -> CompileResult<()> {
        if let MirInstr::ArenaCreate(region_id, initial_capacity) = instr {
            let arena_create_fn = self.funcs["arena_create"];
            let cap_val = self.i64_ty.const_int(*initial_capacity, false);
            
            let arena_ptr = self.builder.build_call(arena_create_fn, &[cap_val.into()], "arena_create_call").unwrap()
                .try_as_basic_value()
                .basic()
                .unwrap()
                .into_pointer_value();
                
            self.arena_ptrs.insert(*region_id, arena_ptr);
            return Ok(());
        }
        unreachable!()
    }
    
    pub(in crate::codegen::instr) fn emit_instr_arena_destroy(&mut self, instr: &MirInstr) -> CompileResult<()> {
        if let MirInstr::ArenaDestroy(region_id) = instr {
            let arena_destroy_fn = self.funcs["arena_destroy"];
            
            let arena_ptr = *self.arena_ptrs.get(region_id).ok_or_else(|| {
                CompileError::Codegen {
                    message: format!("Arena pointer not found for region {}", region_id),
                }
            })?;
            
            self.builder.build_call(arena_destroy_fn, &[arena_ptr.into()], "arena_destroy_call").unwrap();
            return Ok(());
        }
        unreachable!()
    }
}
