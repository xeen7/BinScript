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
    pub(in crate::codegen::instr) fn emit_instr_try_enter(&mut self, instr: &MirInstr) -> CompileResult<()> {
        if let MirInstr::TryEnter { scope_id, catch_target } = instr {
            let pers_fn = self.module.get_function("__bs_personality_v0").unwrap_or_else(|| {
                let ty = self.i32_ty.fn_type(&[], true);
                self.module.add_function("__bs_personality_v0", ty, None)
            });
            let current_fn = self.builder.get_insert_block().unwrap().get_parent().unwrap();
            current_fn.set_personality_function(pers_fn);

            let catch_bb = self.bbs[catch_target];
            self.exception_scope_stack.push((*scope_id, catch_bb));

            if self.gen_state_ptr.is_none() {
                // Zero-cost path: save the current RAII push counter as a
                // compile-time constant. The catch LP will only clean up slots
                // with index >= this value (i.e., objects pushed inside the try body).
                self.catch_raii_indices.insert(catch_bb, self.raii_push_counter);
            } else {
                // Generator fallback: save runtime GUARD_STACK depth
                let get_len_fn = self.module.get_function("__bs_scope_guard_get_len").unwrap_or_else(|| {
                    self.module.add_function("__bs_scope_guard_get_len", self.i32_ty.fn_type(&[], false), None)
                });
                let depth_val = self.builder.build_call(get_len_fn, &[], "catch_depth_val").unwrap().try_as_basic_value().basic().unwrap().into_int_value();
                
                let current_bb = self.builder.get_insert_block().unwrap();
                let entry_bb = current_fn.get_first_basic_block().unwrap();
                if let Some(first_instr) = entry_bb.get_first_instruction() {
                    self.builder.position_before(&first_instr);
                } else {
                    self.builder.position_at_end(entry_bb);
                }
                let depth_ptr = self.builder.build_alloca(self.i32_ty, "catch_depth_ptr").unwrap();
                
                self.builder.position_at_end(current_bb);
                self.builder.build_store(depth_ptr, depth_val).unwrap();
                
                // For generators, store the depth pointer for the runtime flush path
                // (landing_pad.rs will need to handle this via a separate mechanism)
            }
        }
        Ok(())
    }
}
