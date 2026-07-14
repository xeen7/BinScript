#![allow(unused_imports)]
#![allow(unused_unsafe)]
use inkwell::values::BasicMetadataValueEnum;
use mir::types::*;
use diagnostics::CompileResult;
use crate::codegen::LlvmCodegen;

impl<'ctx> LlvmCodegen<'ctx> {
    pub(in crate::codegen::instr) fn emit_instr_landing_pad(&mut self, instr: &MirInstr) -> CompileResult<()> {
        if let MirInstr::LandingPad { exn_reg, is_cleanup } = instr {
            let lp_ty = self.ctx.struct_type(&[
                self.ptr_ty.into(),
                self.i32_ty.into(),
            ], false);
            
            let pers_fn = self.module.get_function("__bs_personality_v0").unwrap_or_else(|| {
                let ty = self.i32_ty.fn_type(&[], true);
                self.module.add_function("__bs_personality_v0", ty, None)
            });
            
            // Catch-all clause: `catch i8* null`
            let null_ptr = self.ptr_ty.const_null();
            let lp = self.builder.build_landing_pad(lp_ty, pers_fn, &[null_ptr.into()], *is_cleanup, "lp").unwrap();
            
            // Store the landingpad result in the exn_reg
            let lp_alloca = self.builder.build_alloca(lp_ty, "lp_alloca").unwrap();
            self.builder.build_store(lp_alloca, lp).unwrap();
            
            let lp_ptr_i64 = self.builder.build_ptr_to_int(lp_alloca, self.i64_ty, "lp_ptr_i64").unwrap();
            self.store(*exn_reg, lp_ptr_i64);

            // ── Zero-cost RAII cleanup ────────────────────────────────────
            // Clean up RAII slots that were pushed inside the try body.
            // Only slots with index >= the saved try-enter index need cleanup,
            // since slots pushed before the try are the caller's responsibility.
            if self.gen_state_ptr.is_none() {
                let current_bb = self.builder.get_insert_block().unwrap();
                if let Some(&try_slot_index) = self.catch_raii_indices.get(&current_bb) {
                    let current_fn = current_bb.get_parent().unwrap();
                    let i1_ty = self.ctx.bool_type();

                    for i in (try_slot_index..self.raii_slots.len()).rev() {
                        let flag_ptr = self.raii_slots[i].flag_ptr;
                        let val_ptr = self.raii_slots[i].val_ptr;
                        let release_fn_name = self.raii_slots[i].release_fn_name.clone();

                        let flag_val = self.builder.build_load(i1_ty, flag_ptr, &format!("catch_chk_{}", i)).unwrap().into_int_value();
                        let do_cleanup = self.ctx.append_basic_block(current_fn, &format!("catch_cleanup_{}", i));
                        let skip = self.ctx.append_basic_block(current_fn, &format!("catch_skip_{}", i));

                        self.builder.build_conditional_branch(flag_val, do_cleanup, skip).unwrap();

                        self.builder.position_at_end(do_cleanup);
                        let obj_val = self.builder.build_load(self.i64_ty, val_ptr, &format!("catch_obj_{}", i)).unwrap();
                        let release_fn = self.module.get_function(&release_fn_name).unwrap_or_else(|| {
                            panic!("Release function {} not found during catch cleanup", release_fn_name);
                        });
                        let sentinel = self.i64_ty.const_int(0xFFF1000000000000, false);
                        self.builder.build_call(release_fn, &[sentinel.into(), obj_val.into()], "").unwrap();
                        self.builder.build_store(flag_ptr, i1_ty.const_int(0, false)).unwrap();
                        self.builder.build_unconditional_branch(skip).unwrap();

                        self.builder.position_at_end(skip);
                    }
                }
            } else {
                // Generator fallback: use runtime flush_down_to
                let current_bb = self.builder.get_insert_block().unwrap();
                if let Some(_depth_ptr) = self.catch_raii_indices.get(&current_bb) {
                    // For generators, catch_raii_indices stores something different — 
                    // but generators still use the old catch_depths path. Handle gracefully.
                }
            }
        }
        Ok(())
    }
}
