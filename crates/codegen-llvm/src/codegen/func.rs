
use mir::types::*;
use diagnostics::CompileResult;

use crate::codegen::LlvmCodegen;

impl<'ctx> LlvmCodegen<'ctx> {
    pub(super) fn emit_function(&mut self, func: &MirFunction) -> CompileResult<()> {
        if func.is_generator || func.is_async {
            self.emit_generator_function(func)
        } else {
            self.emit_normal_function(func)
        }
    }

    fn emit_normal_function(&mut self, func: &MirFunction) -> CompileResult<()> {
        let fv = self.funcs[&func.name];
        self.regs.clear();
        self.bbs.clear();

        // Create LLVM basic blocks.
        for b in &func.blocks {
            let bb = self.ctx.append_basic_block(fv, &format!("bb{}", b.id));
            self.bbs.insert(b.id, bb);
        }

        // Allocas in the entry block.
        let entry = self.bbs[&func.blocks[0].id];
        self.builder.position_at_end(entry);
        
        let regs_array = self.builder.build_alloca(self.i64_ty.array_type(func.next_reg as u32), "regs_array").unwrap();
        let regs_array_ptr = self.builder.build_int_to_ptr(self.builder.build_ptr_to_int(regs_array, self.i64_ty, "").unwrap(), self.ptr_ty, "regs_ptr").unwrap();
        
        for rid in 0..func.next_reg {
            let a = unsafe { self.builder.build_gep(self.i64_ty, regs_array, &[self.i32_ty.const_int(rid as u64, false)], &format!("r{}", rid)).unwrap() };
            self.builder.build_store(a, self.nan.const_undefined()).unwrap();
            self.regs.insert(rid, a);
        }

        // Shadow stack push
        let shadow_frame = self.builder.build_alloca(self.shadow_frame_ty, "shadow_frame").unwrap();
        let num_roots_ptr = self.builder.build_struct_gep(self.shadow_frame_ty, shadow_frame, 1, "num_roots_ptr").unwrap();
        self.builder.build_store(num_roots_ptr, self.i32_ty.const_int(func.next_reg as u64, false)).unwrap();
        let roots_ptr = self.builder.build_struct_gep(self.shadow_frame_ty, shadow_frame, 3, "roots_ptr").unwrap();
        self.builder.build_store(roots_ptr, regs_array_ptr).unwrap();
        let shadow_push_fn = self.module.get_function("__bs_shadow_push").unwrap();
        self.builder.build_call(shadow_push_fn, &[shadow_frame.into()], "shadow_push").unwrap();

        // Store incoming parameters.
        for (i, (reg, _)) in func.params.iter().enumerate() {
            let pv = fv.get_nth_param(i as u32)
                .unwrap_or_else(|| panic!("Function {} has {} params in MIR but LLVM fn has fewer! Failed on param index {}", func.name, func.params.len(), i))
                .into_int_value();
            if let Some(&a) = self.regs.get(reg) {
                self.builder.build_store(a, pv).unwrap();
            }
        }

        // Emit instructions per block.
        for b in &func.blocks {
            self.builder.position_at_end(self.bbs[&b.id]);
            for instr in &b.instrs {
                self.emit_instr(instr)?;
            }
            // Ensure block has a terminator.
            if self.bbs[&b.id].get_terminator().is_none() {
                let shadow_pop_fn = self.module.get_function("__bs_shadow_pop").unwrap();
                self.builder.build_call(shadow_pop_fn, &[], "shadow_pop").unwrap();
                self.builder.build_return(Some(&self.nan.const_undefined())).unwrap();
            }
        }
        Ok(())
    }

    fn emit_generator_function(&mut self, func: &MirFunction) -> CompileResult<()> {
        let fv = self.funcs[&func.name];
        let num_args = func.params.len() as u32;
        let num_locals = func.next_reg;

        let mut struct_fields = vec![
            self.i64_ty.into(), // state_idx
            self.i64_ty.into(), // poll_fn ptr
        ];
        for _ in 0..num_args { struct_fields.push(self.i64_ty.into()); }
        for _ in 0..num_locals { struct_fields.push(self.i64_ty.into()); }

        let state_ty = self.ctx.struct_type(&struct_fields, false);
        let size_val = state_ty.size_of().unwrap();

        let poll_fn_name = format!("{}_poll", func.name);
        let poll_fn_ty = self.i64_ty.fn_type(&[self.ptr_ty.into(), self.i64_ty.into()], false);
        let poll_fv = self.module.add_function(&poll_fn_name, poll_fn_ty, None);

        let wrapper_bb = self.ctx.append_basic_block(fv, "entry");
        self.builder.position_at_end(wrapper_bb);

        let alloc_gen_fn = self.module.get_function("__bs_alloc_generator").unwrap();
        let alloc_call = self.builder.build_call(alloc_gen_fn, &[size_val.into()], "gen_alloc").unwrap();
        let gen_ptr_tagged = alloc_call.try_as_basic_value().basic().unwrap().into_int_value();

        let payload = self.builder.build_and(gen_ptr_tagged, self.i64_ty.const_int(crate::nan_box::PAYLOAD_MASK, false), "payload").unwrap();
        let state_ptr = self.builder.build_int_to_ptr(payload, self.ptr_ty, "state_ptr").unwrap();

        let state_idx_ptr = self.builder.build_struct_gep(state_ty, state_ptr, 0, "state_idx_ptr").unwrap();
        self.builder.build_store(state_idx_ptr, self.i64_ty.const_int(0, false)).unwrap();

        let poll_fn_ptr = poll_fv.as_global_value().as_pointer_value();
        let poll_fn_i64 = self.builder.build_ptr_to_int(poll_fn_ptr, self.i64_ty, "poll_fn_i64").unwrap();
        let poll_slot_ptr = self.builder.build_struct_gep(state_ty, state_ptr, 1, "poll_slot").unwrap();
        self.builder.build_store(poll_slot_ptr, poll_fn_i64).unwrap();

        for (i, _) in func.params.iter().enumerate() {
            let arg_val = fv.get_nth_param(i as u32).unwrap().into_int_value();
            let arg_slot = self.builder.build_struct_gep(state_ty, state_ptr, 2 + i as u32, "arg_slot").unwrap();
            self.builder.build_store(arg_slot, arg_val).unwrap();
        }

        if func.is_async && !func.is_generator {
            let drive_fn = self.module.get_function("__bs_async_drive").unwrap();
            let drive_call = self.builder.build_call(drive_fn, &[gen_ptr_tagged.into()], "drive_async").unwrap();
            let promise_ptr = drive_call.try_as_basic_value().basic().unwrap().into_int_value();
            self.builder.build_return(Some(&promise_ptr)).unwrap();
        } else {
            self.builder.build_return(Some(&gen_ptr_tagged)).unwrap();
        }
        self.regs.clear();
        self.bbs.clear();
        self.resume_blocks.clear();
        self.gen_state_ty = Some(state_ty);
        self.gen_num_args = num_args;

        let poll_entry = self.ctx.append_basic_block(poll_fv, "entry");
        self.builder.position_at_end(poll_entry);

        let state_arg = poll_fv.get_nth_param(0).unwrap().into_pointer_value();
        let sent_val_arg = poll_fv.get_nth_param(1).unwrap().into_int_value();
        self.gen_state_ptr = Some(state_arg);
        self.gen_sent_val = Some(sent_val_arg);

        let regs_array = self.builder.build_alloca(self.i64_ty.array_type(func.next_reg as u32), "regs_array").unwrap();
        let regs_array_ptr = self.builder.build_int_to_ptr(self.builder.build_ptr_to_int(regs_array, self.i64_ty, "").unwrap(), self.ptr_ty, "regs_ptr").unwrap();

        for (i, (reg, _)) in func.params.iter().enumerate() {
            let arg_slot = self.builder.build_struct_gep(state_ty, state_arg, 2 + i as u32, "arg_slot").unwrap();
            let loaded = self.builder.build_load(self.i64_ty, arg_slot, "loaded_arg").unwrap().into_int_value();
            let a = unsafe { self.builder.build_gep(self.i64_ty, regs_array, &[self.i32_ty.const_int(*reg as u64, false)], &format!("r{}", reg)).unwrap() };
            self.builder.build_store(a, loaded).unwrap();
            self.regs.insert(*reg, a);
        }

        for rid in 0..func.next_reg {
            if !self.regs.contains_key(&rid) {
                let a = unsafe { self.builder.build_gep(self.i64_ty, regs_array, &[self.i32_ty.const_int(rid as u64, false)], &format!("r{}", rid)).unwrap() };
                self.builder.build_store(a, self.nan.const_undefined()).unwrap();
                self.regs.insert(rid, a);
            }
        }

        // Shadow stack push
        let shadow_frame = self.builder.build_alloca(self.shadow_frame_ty, "shadow_frame").unwrap();
        let num_roots_ptr = self.builder.build_struct_gep(self.shadow_frame_ty, shadow_frame, 1, "num_roots_ptr").unwrap();
        self.builder.build_store(num_roots_ptr, self.i32_ty.const_int(func.next_reg as u64, false)).unwrap();
        let roots_ptr = self.builder.build_struct_gep(self.shadow_frame_ty, shadow_frame, 3, "roots_ptr").unwrap();
        self.builder.build_store(roots_ptr, regs_array_ptr).unwrap();
        let shadow_push_fn = self.module.get_function("__bs_shadow_push").unwrap();
        self.builder.build_call(shadow_push_fn, &[shadow_frame.into()], "shadow_push").unwrap();

        for b in &func.blocks {
            let bb = self.ctx.append_basic_block(poll_fv, &format!("bb{}", b.id));
            self.bbs.insert(b.id, bb);
        }

        let done_bb = self.ctx.append_basic_block(poll_fv, "done");

        for i in 0..func.num_yield_points {
            let rbb = self.ctx.append_basic_block(poll_fv, &format!("resume_{}", i));
            self.resume_blocks.insert(i, rbb);
        }

        let state_idx_ptr = self.builder.build_struct_gep(state_ty, state_arg, 0, "state_idx_ptr").unwrap();
        let state_val = self.builder.build_load(self.i64_ty, state_idx_ptr, "state_val").unwrap().into_int_value();

        let mut switch_cases = Vec::new();
        switch_cases.push((self.i64_ty.const_int(0, false), self.bbs[&func.blocks[0].id]));
        for i in 0..func.num_yield_points {
            switch_cases.push((self.i64_ty.const_int((i + 1) as u64, false), self.resume_blocks[&i]));
        }

        self.builder.build_switch(state_val, done_bb, &switch_cases).unwrap();

        self.builder.position_at_end(done_bb);
        let shadow_pop_fn = self.module.get_function("__bs_shadow_pop").unwrap();
        self.builder.build_call(shadow_pop_fn, &[], "shadow_pop").unwrap();
        self.builder.build_return(Some(&self.nan.const_undefined())).unwrap();

        for b in &func.blocks {
            self.builder.position_at_end(self.bbs[&b.id]);
            for instr in &b.instrs {
                self.emit_instr(instr)?;
            }
            if self.bbs[&b.id].get_terminator().is_none() {
                if let Some(state_ptr) = self.gen_state_ptr {
                    let state_idx_ptr = self.builder.build_struct_gep(state_ty, state_ptr, 0, "state_idx_ptr").unwrap();
                    self.builder.build_store(state_idx_ptr, self.i64_ty.const_all_ones()).unwrap();
                }
                let shadow_pop_fn = self.module.get_function("__bs_shadow_pop").unwrap();
                self.builder.build_call(shadow_pop_fn, &[], "shadow_pop").unwrap();
                self.builder.build_return(Some(&self.nan.const_undefined())).unwrap();
            }
        }

        self.gen_state_ptr = None;
        self.gen_sent_val = None;
        self.gen_state_ty = None;
        self.resume_blocks.clear();
        self.gen_num_args = 0;

        Ok(())
    }

    pub(super) fn emit_main(&mut self, body: &MirFunction) -> CompileResult<()> {
        let ft = self.i32_ty.fn_type(&[], false);
        let main_fn = self.module.add_function("main", ft, None);

        self.regs.clear();
        self.bbs.clear();

        for b in &body.blocks {
            let bb = self.ctx.append_basic_block(main_fn, &format!("bb{}", b.id));
            self.bbs.insert(b.id, bb);
        }

        let entry = self.bbs[&body.blocks[0].id];
        self.builder.position_at_end(entry);

        let array_ty = self.i64_ty.array_type(body.next_reg as u32);
        let regs_array = self.builder.build_alloca(array_ty, "regs_array").unwrap();
        let regs_array_ptr = self.builder.build_ptr_to_int(regs_array, self.i64_ty, "").unwrap();
        let regs_array_ptr = self.builder.build_int_to_ptr(regs_array_ptr, self.ctx.ptr_type(inkwell::AddressSpace::default()), "regs_ptr").unwrap();

        for rid in 0..body.next_reg {
            let a = unsafe { self.builder.build_gep(array_ty, regs_array, &[self.i32_ty.const_int(0, false), self.i32_ty.const_int(rid as u64, false)], &format!("r{}", rid)).unwrap() };
            self.builder.build_store(a, self.nan.const_undefined()).unwrap();
            self.regs.insert(rid, a);
        }

        let shadow_frame = self.builder.build_alloca(self.shadow_frame_ty, "shadow_frame").unwrap();
        let num_roots_ptr = self.builder.build_struct_gep(self.shadow_frame_ty, shadow_frame, 1, "num_roots_ptr").unwrap();
        self.builder.build_store(num_roots_ptr, self.i32_ty.const_int(body.next_reg as u64, false)).unwrap();
        let roots_ptr = self.builder.build_struct_gep(self.shadow_frame_ty, shadow_frame, 3, "roots_ptr").unwrap();
        self.builder.build_store(roots_ptr, regs_array_ptr).unwrap();
        
        let shadow_push_fn = self.module.get_function("__bs_shadow_push").unwrap();
        self.builder.build_call(shadow_push_fn, &[shadow_frame.into()], "").unwrap();

        for b in &body.blocks {
            self.builder.position_at_end(self.bbs[&b.id]);
            for instr in &b.instrs {
                // Skip Return instructions — main always returns i32 0.
                if matches!(instr, MirInstr::Return(_)) {
                    continue;
                }
                self.emit_instr(instr)?;
            }
            if self.bbs[&b.id].get_terminator().is_none() {
                let drain_fn = self.module.get_function("__bs_drain_microtasks").unwrap();
                self.builder.build_call(drain_fn, &[], "drain").unwrap();

                let shadow_pop_fn = self.module.get_function("__bs_shadow_pop").unwrap();
                self.builder.build_call(shadow_pop_fn, &[], "").unwrap();

                self.builder
                    .build_return(Some(&self.i32_ty.const_int(0, false)))
                    .unwrap();
            }
        }
        Ok(())
    }
}
