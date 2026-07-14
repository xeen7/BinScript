use inkwell::values::BasicMetadataValueEnum;
use inkwell::IntPredicate;
use mir::types::*;
use diagnostics::{CompileError, CompileResult};

use crate::codegen::LlvmCodegen;

impl<'ctx> LlvmCodegen<'ctx> {
    pub(in crate::codegen::instr) fn emit_instr_rc_inc(&mut self, instr: &MirInstr) -> CompileResult<()> {
        if let MirInstr::RcInc(reg) = instr {
            let val = self.val(&MirOperand::Reg(*reg))?;
            self.emit_call_circ_fn(val, "circ_inc")?;
        }
        Ok(())
    }

    pub(in crate::codegen::instr) fn emit_instr_rc_dec(&mut self, instr: &MirInstr) -> CompileResult<()> {
        if let MirInstr::RcDec(reg) = instr {
            let val = self.val(&MirOperand::Reg(*reg))?;
            self.emit_call_circ_fn(val, "circ_dec")?;
            self.deferred_clears.push(*reg);
            // Clear the register to TAG_UNDEFINED so that Generator trace_fn/drop_fn
            // and Cycle Collector don't double-free or trace a garbage pointer.
            // We defer this clear to just before the next instruction or inside terminators
            // so terminators can still read the original value.
        }
        Ok(())
    }

    pub(in crate::codegen::instr) fn emit_instr_rc_inc_deferred(&mut self, instr: &MirInstr) -> CompileResult<()> {
        if let MirInstr::RcIncDeferred(reg) = instr {
            let val = self.val(&MirOperand::Reg(*reg))?;
            self.emit_call_circ_fn(val, "__bs_rc_inc_deferred")?;
        }
        Ok(())
    }

    pub(in crate::codegen::instr) fn emit_instr_rc_dec_deferred(&mut self, instr: &MirInstr) -> CompileResult<()> {
        if let MirInstr::RcDecDeferred(reg) = instr {
            let val = self.val(&MirOperand::Reg(*reg))?;
            self.emit_call_circ_fn(val, "__bs_rc_dec_deferred")?;
        }
        Ok(())
    }

    pub(in crate::codegen::instr) fn emit_instr_flush_rc_delta(&mut self) -> CompileResult<()> {
        let fn_name = "__bs_rc_flush";
        let flush_fn = self.module.get_function(fn_name).unwrap_or_else(|| {
            let ty = self.void_ty.fn_type(&[], false);
            self.module.add_function(fn_name, ty, None)
        });
        self.builder.build_call(flush_fn, &[], "call_rc_flush").unwrap();
        Ok(())
    }

    pub(crate) fn emit_call_circ_fn(&mut self, val: inkwell::values::IntValue<'ctx>, fn_name: &str) -> CompileResult<()> {
        let tag = self.builder.build_right_shift(
            val, self.i64_ty.const_int(48, false), false, "rc_tag"
        ).unwrap();

        let is_obj = self.builder.build_int_compare(
            IntPredicate::EQ, tag, self.i64_ty.const_int(0xFFF6, false), "is_obj"
        ).unwrap();
        let is_closure = self.builder.build_int_compare(
            IntPredicate::EQ, tag, self.i64_ty.const_int(0xFFF9, false), "is_closure"
        ).unwrap();
        let is_gen = self.builder.build_int_compare(
            IntPredicate::EQ, tag, self.i64_ty.const_int(0xFFFA, false), "is_gen"
        ).unwrap();
        let is_array = self.builder.build_int_compare(
            IntPredicate::EQ, tag, self.i64_ty.const_int(0xFFFB, false), "is_array"
        ).unwrap();
        let is_obj_or_closure = self.builder.build_or(is_obj, is_closure, "is_obj_or_closure").unwrap();
        let is_gen_or_array = self.builder.build_or(is_gen, is_array, "is_gen_or_array").unwrap();
        let is_circ = self.builder.build_or(is_obj_or_closure, is_gen_or_array, "is_circ").unwrap();

        let current_fn = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let do_call_bb = self.ctx.append_basic_block(current_fn, &format!("do_{}", fn_name));
        let skip_call_bb = self.ctx.append_basic_block(current_fn, &format!("skip_{}", fn_name));
        self.builder.build_conditional_branch(is_circ, do_call_bb, skip_call_bb).unwrap();

        self.builder.position_at_end(do_call_bb);
        let mask = self.i64_ty.const_int(0x0000_FFFF_FFFF_FFFF, false);
        let raw_ptr_i64 = self.builder.build_and(val, mask, "unbox_ptr").unwrap();
        let raw_ptr = self.builder.build_int_to_ptr(raw_ptr_i64, self.ptr_ty, "ptr").unwrap();
        
        /*
        // Add printf to see if do_call_bb is executed!
        let printf_fn = self.module.get_function("printf").unwrap_or_else(|| {
            let ty = self.i32_ty.fn_type(&[self.ptr_ty.into()], true);
            self.module.add_function("printf", ty, None)
        });
        let format_str = self.builder.build_global_string_ptr(&format!("do_call_bb for {} executed! ptr: %p\\n", fn_name), "do_call_fmt").unwrap();
        self.builder.build_call(printf_fn, &[format_str.as_pointer_value().into(), raw_ptr.into()], "printf").unwrap();
        */

        let offset = self.i32_ty.const_int(24_u64.wrapping_neg(), true);
        let header_ptr = unsafe { self.builder.build_in_bounds_gep(self.i8_ty, raw_ptr, &[offset], "header_ptr").unwrap() };
        
        let circ_fn = self.module.get_function(fn_name).unwrap_or_else(|| {
            let ty = self.void_ty.fn_type(&[self.ptr_ty.into()], false);
            self.module.add_function(fn_name, ty, None)
        });
        
        self.builder.build_call(circ_fn, &[header_ptr.into()], &format!("call_{}", fn_name)).unwrap();
        self.builder.build_unconditional_branch(skip_call_bb).unwrap();

        self.builder.position_at_end(skip_call_bb);
        Ok(())
    }

    pub(in crate::codegen::instr) fn emit_instr_drop(&mut self, instr: &MirInstr) -> CompileResult<()> {
        if let MirInstr::Drop(reg) = instr {
            let val = self.val(&MirOperand::Reg(*reg))?;
            
            // Extract the tag: (val >> 48)
            let shift = self.i64_ty.const_int(48, false);
            let tag = self.builder.build_right_shift(val, shift, false, "rc_tag").unwrap();
            
            let tag_owned = self.i64_ty.const_int(0xFFFC, false);
            let tag_owned_closure = self.i64_ty.const_int(0x7FF9, false);
            let tag_owned_array = self.i64_ty.const_int(0x7FFB, false);
            let tag_owned_string = self.i64_ty.const_int(0x7FF7, false);
            
            let is_owned_obj = self.builder.build_int_compare(IntPredicate::EQ, tag, tag_owned, "is_owned_obj").unwrap();
            let is_owned_clo = self.builder.build_int_compare(IntPredicate::EQ, tag, tag_owned_closure, "is_owned_clo").unwrap();
            let is_owned_arr = self.builder.build_int_compare(IntPredicate::EQ, tag, tag_owned_array, "is_owned_arr").unwrap();
            let is_owned_str = self.builder.build_int_compare(IntPredicate::EQ, tag, tag_owned_string, "is_owned_str").unwrap();
            
            let is_owned = self.builder.build_or(is_owned_obj, is_owned_clo, "is_owned_1").unwrap();
            let is_owned = self.builder.build_or(is_owned, is_owned_arr, "is_owned").unwrap();

            let current_fn = self.builder.get_insert_block().unwrap().get_parent().unwrap();
            let owned_block = self.ctx.append_basic_block(current_fn, "drop_owned");
            let str_block = self.ctx.append_basic_block(current_fn, "drop_str");
            let cont_block = self.ctx.append_basic_block(current_fn, "drop_cont");
            
            // Branch to str_block if string, else check other owned types
            let non_str_check_bb = self.ctx.append_basic_block(current_fn, "check_owned_non_str");
            self.builder.build_conditional_branch(is_owned_str, str_block, non_str_check_bb).unwrap();
            
            // --- check_owned_non_str ---
            self.builder.position_at_end(non_str_check_bb);
            self.builder.build_conditional_branch(is_owned, owned_block, cont_block).unwrap();
            
            // Extract raw ptr for both blocks
            let mask = self.i64_ty.const_int(0x0000_FFFF_FFFF_FFFF, false);
            
            // --- drop_owned ---
            self.builder.position_at_end(owned_block);
            let raw_ptr_i64_1 = self.builder.build_and(val, mask, "unbox_ptr_1").unwrap();
            let raw_ptr_1 = self.builder.build_int_to_ptr(raw_ptr_i64_1, self.ptr_ty, "ptr_1").unwrap();
            let drop_owned_fn = self.funcs["__bs_drop_owned"];
            self.builder.build_call(drop_owned_fn, &[raw_ptr_1.into()], "call_drop_owned").unwrap();
            self.builder.build_unconditional_branch(cont_block).unwrap();
            
            // --- drop_str ---
            self.builder.position_at_end(str_block);
            let raw_ptr_i64_2 = self.builder.build_and(val, mask, "unbox_ptr_2").unwrap();
            let raw_ptr_2 = self.builder.build_int_to_ptr(raw_ptr_i64_2, self.ptr_ty, "ptr_2").unwrap();
            let free_fn = self.funcs["free"];
            self.builder.build_call(free_fn, &[raw_ptr_2.into()], "call_free_str").unwrap();
            self.builder.build_unconditional_branch(cont_block).unwrap();

            // --- cont_block ---
            self.builder.position_at_end(cont_block);
        }
        Ok(())
    }

    pub(in crate::codegen::instr) fn emit_instr_drop_stack(&mut self, instr: &MirInstr) -> CompileResult<()> {
        if let MirInstr::DropStack(reg) = instr {
            self.emit_call_drop_fn_only(*reg)?;
        }
        Ok(())
    }

    pub(in crate::codegen::instr) fn emit_instr_call_drop_fn_only(&mut self, instr: &MirInstr) -> CompileResult<()> {
        if let MirInstr::CallDropFnOnly(reg) = instr {
            self.emit_call_drop_fn_only(*reg)?;
        }
        Ok(())
    }

    fn emit_call_drop_fn_only(&mut self, reg: MirReg) -> CompileResult<()> {
        let val = self.val(&MirOperand::Reg(reg))?;
        
        let mask = self.i64_ty.const_int(0x0000_FFFF_FFFF_FFFF, false);
        let raw_ptr_i64 = self.builder.build_and(val, mask, "unbox_ptr").unwrap();
        let raw_ptr = self.builder.build_int_to_ptr(raw_ptr_i64, self.ptr_ty, "ptr").unwrap();
        
        // Load vtable_ptr.
        let vtable_ptr_ptr = raw_ptr; // it's already a pointer to the object, which starts with vtable_ptr
        let vtable_ptr = self.builder.build_load(self.ptr_ty, vtable_ptr_ptr, "vtable_ptr").unwrap().into_pointer_value();
        
        // drop_fn is at byte offset 40 in VTable struct:
        //   parent(8) + name(8) + shape_id(8) + fields_count(8) + field_names(8) = 40
        let drop_fn_offset = self.i32_ty.const_int(40, false);
        let drop_fn_ptr_ptr = unsafe { self.builder.build_in_bounds_gep(self.i8_ty, vtable_ptr, &[drop_fn_offset], "drop_fn_ptr_ptr").unwrap() };
        
        let drop_fn_ptr = self.builder.build_load(self.ptr_ty, drop_fn_ptr_ptr, "drop_fn_ptr").unwrap().into_pointer_value();
        
        // Check if drop_fn is not null
        let is_not_null = self.builder.build_is_not_null(drop_fn_ptr, "drop_not_null").unwrap();
        
        let current_fn = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let drop_block = self.ctx.append_basic_block(current_fn, "do_drop");
        let cont_block = self.ctx.append_basic_block(current_fn, "cont_drop");
        
        self.builder.build_conditional_branch(is_not_null, drop_block, cont_block).unwrap();
        
        // In drop_block: call drop_fn(raw_ptr)
        self.builder.position_at_end(drop_block);
        let drop_fn_type = self.void_ty.fn_type(&[self.ptr_ty.into()], false);
        self.builder.build_indirect_call(drop_fn_type, drop_fn_ptr, &[raw_ptr.into()], "call_drop_fn").unwrap();
        self.builder.build_unconditional_branch(cont_block).unwrap();
        
        // In cont_block: do nothing, do NOT call free
        self.builder.position_at_end(cont_block);
        
        Ok(())
    }

    pub(in crate::codegen::instr) fn emit_instr_store_shared_field(&mut self, instr: &MirInstr) -> CompileResult<()> {
        if let MirInstr::StoreSharedField(obj_reg, index, val_operand, is_moved) = instr {
            // A store to a shared field is a cross-thread sharing point.
            // We must promote the object to globally shared by flushing its local_rc.
            let val = self.val(val_operand)?;
            self.emit_call_circ_fn(val, "circ_promote")?;

            // Now perform the normal field store
            self.emit_instr_store_field(&MirInstr::StoreField(*obj_reg, *index, val_operand.clone()))?;
            
            // Note: Since Ownership Inference already injects RcInc and RcDec
            // around the old/new values respectively, we don't need to do it here.
        }
        Ok(())
    }

    pub(in crate::codegen::instr) fn emit_instr_force_owned_tag(&mut self, instr: &mir::MirInstr) -> CompileResult<()> {
        if let mir::MirInstr::ForceOwnedTag(reg) = instr {
            let val = self.val(&mir::MirOperand::Reg(*reg))?;
            let mask = self.i64_ty.const_int(0x7FFF_FFFF_FFFF_FFFF, false);
            let new_val = self.builder.build_and(val, mask, "force_owned_tag").unwrap();
            self.store(*reg, new_val);
        }
        Ok(())
    }
}
