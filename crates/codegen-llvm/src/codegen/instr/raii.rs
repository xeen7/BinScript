use inkwell::IntPredicate;
use mir::types::{MirOperand, MirReg};
use diagnostics::CompileResult;

use crate::codegen::LlvmCodegen;

impl<'ctx> LlvmCodegen<'ctx> {
    pub(in crate::codegen::instr) fn emit_instr_scope_guard_push(
        &mut self,
        scope_id: u32,
        reg: MirReg,
        release_fn_name: &str,
    ) -> CompileResult<()> {
        // For generator functions, fall back to the runtime stack
        if self.gen_state_ptr.is_some() {
            return self.emit_instr_scope_guard_push_runtime(scope_id, reg, release_fn_name);
        }

        // Zero-cost path: set the liveness flag and store the object value
        if let Some(&slot_idx) = self.raii_reg_to_slot.get(&reg) {
            let flag_ptr = self.raii_slots[slot_idx].flag_ptr;
            let val_ptr = self.raii_slots[slot_idx].val_ptr;
            let i1_ty = self.ctx.bool_type();
            self.builder.build_store(flag_ptr, i1_ty.const_int(1, false)).unwrap();
            let val = self.val(&MirOperand::Reg(reg))?;
            self.builder.build_store(val_ptr, val).unwrap();
        }
        self.raii_push_counter += 1;
        Ok(())
    }

    pub(in crate::codegen::instr) fn emit_instr_scope_guard_cancel(
        &mut self,
        scope_id: u32,
        reg: MirReg,
    ) -> CompileResult<()> {
        // For generator functions, fall back to the runtime stack
        if self.gen_state_ptr.is_some() {
            return self.emit_instr_scope_guard_cancel_runtime(scope_id, reg);
        }

        // Zero-cost path: clear the liveness flag
        if let Some(&slot_idx) = self.raii_reg_to_slot.get(&reg) {
            let slot = &self.raii_slots[slot_idx];
            let i1_ty = self.ctx.bool_type();
            self.builder.build_store(slot.flag_ptr, i1_ty.const_int(0, false)).unwrap();
        }
        Ok(())
    }

    pub(in crate::codegen::instr) fn emit_instr_scope_guard_flush_to(
        &mut self,
        target_scope: u32,
    ) -> CompileResult<()> {
        // For generator functions, fall back to the runtime stack
        if self.gen_state_ptr.is_some() {
            return self.emit_instr_scope_guard_flush_to_runtime(target_scope);
        }

        // Zero-cost path: emit inline conditional destructor calls
        // Check each RAII slot in reverse order. If the flag is true, call the
        // release function and clear the flag.
        let current_fn = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let i1_ty = self.ctx.bool_type();

        for i in (0..self.raii_slots.len()).rev() {
            let flag_ptr = self.raii_slots[i].flag_ptr;
            let val_ptr = self.raii_slots[i].val_ptr;
            let release_fn_name = self.raii_slots[i].release_fn_name.clone();

            let flag_val = self.builder.build_load(i1_ty, flag_ptr, &format!("raii_chk_{}", i)).unwrap().into_int_value();
            let flush_bb = self.ctx.append_basic_block(current_fn, &format!("raii_flush_{}", i));
            let skip_bb = self.ctx.append_basic_block(current_fn, &format!("raii_skip_{}", i));

            self.builder.build_conditional_branch(flag_val, flush_bb, skip_bb).unwrap();

            self.builder.position_at_end(flush_bb);
            let obj_val = self.builder.build_load(self.i64_ty, val_ptr, &format!("raii_obj_{}", i)).unwrap();
            let release_fn = self.module.get_function(&release_fn_name).unwrap_or_else(|| {
                panic!("Release function {} not found during ScopeGuardFlushTo", release_fn_name);
            });
            // Call release_fn(sentinel, obj_val) — sentinel 0xFFF1... signals cleanup
            let sentinel = self.i64_ty.const_int(0xFFF1000000000000, false);
            self.builder.build_call(release_fn, &[sentinel.into(), obj_val.into()], "").unwrap();
            self.builder.build_store(flag_ptr, i1_ty.const_int(0, false)).unwrap();
            self.builder.build_unconditional_branch(skip_bb).unwrap();

            self.builder.position_at_end(skip_bb);
        }
        Ok(())
    }

    // ── Runtime fallback for generator functions ───────────────────────────

    fn emit_instr_scope_guard_push_runtime(
        &mut self,
        _scope_id: u32,
        reg: MirReg,
        release_fn_name: &str,
    ) -> CompileResult<()> {
        let val = self.val(&MirOperand::Reg(reg))?;
        let scope_val = self.i32_ty.const_int(1, false);
        let func = self.module.get_function(release_fn_name).unwrap_or_else(|| {
            panic!("Release function {} not found", release_fn_name);
        });
        let func_ptr = func.as_global_value().as_pointer_value();
        let push_fn = self.funcs["__bs_scope_guard_push"];
        self.builder.build_call(push_fn, &[scope_val.into(), val.into(), func_ptr.into()], "").unwrap();
        Ok(())
    }

    fn emit_instr_scope_guard_cancel_runtime(
        &mut self,
        _scope_id: u32,
        reg: MirReg,
    ) -> CompileResult<()> {
        let val = self.val(&MirOperand::Reg(reg))?;
        let scope_val = self.i32_ty.const_int(1, false);
        let cancel_fn = self.funcs["__bs_scope_guard_cancel"];
        self.builder.build_call(cancel_fn, &[self.frame_base.unwrap().into(), scope_val.into(), val.into()], "").unwrap();
        Ok(())
    }

    fn emit_instr_scope_guard_flush_to_runtime(
        &mut self,
        target_scope: u32,
    ) -> CompileResult<()> {
        let target_scope_val = self.i32_ty.const_int(target_scope as u64, false);
        let flush_fn = self.funcs["__bs_scope_guard_flush_to"];
        self.builder.build_call(flush_fn, &[self.frame_base.unwrap().into(), target_scope_val.into()], "").unwrap();
        Ok(())
    }
}
