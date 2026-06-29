#![allow(unused_imports)]
#![allow(unused_unsafe)]
use inkwell::values::{BasicValueEnum, BasicMetadataValueEnum, FunctionValue, InstructionValue, CallSiteValue};
use inkwell::basic_block::BasicBlock as LlvmBB;
use diagnostics::{CompileError, CompileResult};
use mir::types::*;
use crate::codegen::LlvmCodegen;

impl<'ctx> LlvmCodegen<'ctx> {
    pub(in crate::codegen::instr) fn emit_call_with_invoke(
        &mut self,
        callee: FunctionValue<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
        name: &str,
    ) -> CompileResult<CallSiteValue<'ctx>> {
        // If we have an active exception scope, we MUST use invoke
        if let Some(&(scope_id, catch_bb)) = self.exception_scope_stack.last() {
            let current_fn = self.builder.get_insert_block().unwrap().get_parent().unwrap();
            let normal_bb = self.ctx.append_basic_block(current_fn, "invoke_normal");
            
            let basic_args = Self::to_basic_args(args);
            let invoke = self.builder.build_invoke(callee, &basic_args, normal_bb, catch_bb, name).unwrap();
            self.builder.position_at_end(normal_bb);
            Ok(invoke)
        } else if !self.raii_slots.is_empty() && self.gen_state_ptr.is_none() {
            // Outside any try block, but RAII objects may be live.
            // Route to a cleanup-only landing pad that destroys flagged
            // objects and resumes unwinding.
            let cleanup_bb = self.get_or_create_cleanup_bb();
            let current_fn = self.builder.get_insert_block().unwrap().get_parent().unwrap();
            let normal_bb = self.ctx.append_basic_block(current_fn, "invoke_normal");
            
            let basic_args = Self::to_basic_args(args);
            let invoke = self.builder.build_invoke(callee, &basic_args, normal_bb, cleanup_bb, name).unwrap();
            self.builder.position_at_end(normal_bb);
            Ok(invoke)
        } else {
            let call = self.builder.build_call(callee, args, name).unwrap();
            Ok(call)
        }
    }

    pub(in crate::codegen::instr) fn emit_indirect_call_with_invoke(
        &mut self,
        fn_ty: inkwell::types::FunctionType<'ctx>,
        fn_ptr: inkwell::values::PointerValue<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
        name: &str,
    ) -> CompileResult<CallSiteValue<'ctx>> {
        // If we have an active exception scope, we MUST use invoke
        if let Some(&(scope_id, catch_bb)) = self.exception_scope_stack.last() {
            let current_fn = self.builder.get_insert_block().unwrap().get_parent().unwrap();
            let normal_bb = self.ctx.append_basic_block(current_fn, "invoke_normal");
            
            let basic_args = Self::to_basic_args(args);
            let invoke = self.builder.build_indirect_invoke(fn_ty, fn_ptr, &basic_args, normal_bb, catch_bb, name).unwrap();
            self.builder.position_at_end(normal_bb);
            Ok(invoke)
        } else if !self.raii_slots.is_empty() && self.gen_state_ptr.is_none() {
            let cleanup_bb = self.get_or_create_cleanup_bb();
            let current_fn = self.builder.get_insert_block().unwrap().get_parent().unwrap();
            let normal_bb = self.ctx.append_basic_block(current_fn, "invoke_normal");
            
            let basic_args = Self::to_basic_args(args);
            let invoke = self.builder.build_indirect_invoke(fn_ty, fn_ptr, &basic_args, normal_bb, cleanup_bb, name).unwrap();
            self.builder.position_at_end(normal_bb);
            Ok(invoke)
        } else {
            let call = self.builder.build_indirect_call(fn_ty, fn_ptr, args, name).unwrap();
            Ok(call)
        }
    }

    /// Convert BasicMetadataValueEnum args to BasicValueEnum for invoke.
    fn to_basic_args(args: &[BasicMetadataValueEnum<'ctx>]) -> Vec<BasicValueEnum<'ctx>> {
        args.iter().map(|a| {
            if a.is_int_value() {
                BasicValueEnum::IntValue(a.into_int_value())
            } else if a.is_float_value() {
                BasicValueEnum::FloatValue(a.into_float_value())
            } else if a.is_pointer_value() {
                BasicValueEnum::PointerValue(a.into_pointer_value())
            } else {
                unreachable!()
            }
        }).collect()
    }

    /// Get or lazily create a cleanup-only landing pad for unwinding through
    /// this function when outside any try block. The LP checks all RAII flags,
    /// calls destructors for live objects, then resumes unwinding.
    fn get_or_create_cleanup_bb(&mut self) -> LlvmBB<'ctx> {
        if let Some(bb) = self.raii_cleanup_bb {
            return bb;
        }

        let current_bb = self.builder.get_insert_block().unwrap();
        let current_fn = current_bb.get_parent().unwrap();

        // Set personality function on this function
        let pers_fn = self.module.get_function("__bs_personality_v0").unwrap_or_else(|| {
            let ty = self.i32_ty.fn_type(&[], true);
            self.module.add_function("__bs_personality_v0", ty, None)
        });
        current_fn.set_personality_function(pers_fn);

        let cleanup_bb = self.ctx.append_basic_block(current_fn, "raii_cleanup");
        self.builder.position_at_end(cleanup_bb);

        // Landing pad: cleanup only (no catch), so the unwinder continues
        // searching for a handler after running our destructors.
        let lp_ty = self.ctx.struct_type(&[self.ptr_ty.into(), self.i32_ty.into()], false);
        let lp = self.builder.build_landing_pad(lp_ty, pers_fn, &[], true, "cleanup_lp").unwrap();

        // Check each RAII flag in reverse order and call destructors
        let i1_ty = self.ctx.bool_type();
        for i in (0..self.raii_slots.len()).rev() {
            let flag_ptr = self.raii_slots[i].flag_ptr;
            let val_ptr = self.raii_slots[i].val_ptr;
            let release_fn_name = self.raii_slots[i].release_fn_name.clone();

            let flag_val = self.builder.build_load(i1_ty, flag_ptr, &format!("cleanup_chk_{}", i)).unwrap().into_int_value();
            let do_cleanup = self.ctx.append_basic_block(current_fn, &format!("cleanup_do_{}", i));
            let next = self.ctx.append_basic_block(current_fn, &format!("cleanup_next_{}", i));

            self.builder.build_conditional_branch(flag_val, do_cleanup, next).unwrap();

            self.builder.position_at_end(do_cleanup);
            let obj_val = self.builder.build_load(self.i64_ty, val_ptr, &format!("cleanup_obj_{}", i)).unwrap();
            let release_fn = self.module.get_function(&release_fn_name).unwrap_or_else(|| {
                panic!("Release function {} not found during cleanup LP creation", release_fn_name);
            });
            let sentinel = self.i64_ty.const_int(0xFFF1000000000000, false);
            self.builder.build_call(release_fn, &[sentinel.into(), obj_val.into()], "").unwrap();
            self.builder.build_store(flag_ptr, i1_ty.const_int(0, false)).unwrap();
            self.builder.build_unconditional_branch(next).unwrap();

            self.builder.position_at_end(next);
        }

        // Resume unwinding after all cleanup is done
        self.builder.build_resume(lp).unwrap();

        // Restore builder position
        self.builder.position_at_end(current_bb);

        self.raii_cleanup_bb = Some(cleanup_bb);
        cleanup_bb
    }
}
