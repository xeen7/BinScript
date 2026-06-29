
use mir::types::*;
use diagnostics::CompileResult;

use crate::codegen::LlvmCodegen;
use inkwell::values::FunctionValue;

impl<'ctx> LlvmCodegen<'ctx> {
    pub(super) fn emit_function(&mut self, func: &MirFunction) -> CompileResult<()> {
        // println!("Emitting codegen for func: {}", func.name);
        if func.is_generator || func.is_async {
            self.emit_generator_function(func)
        } else {
            self.emit_normal_function(func)
        }
    }

    fn emit_normal_function(&mut self, func: &MirFunction) -> CompileResult<()> {
        for (i, b) in func.blocks.iter().enumerate() {
            // println!("Codegen Block {}:", i);
            for instr in &b.instrs {
                // println!("  {:?}", instr);
            }
        }
        let fv = self.funcs[&func.name];
        self.regs.clear();
        self.bbs.clear();

        // Give the function the uwtable attribute so it generates .eh_frame
        let uwtable_kind = inkwell::attributes::Attribute::get_named_enum_kind_id("uwtable");
        let uwtable_attr = self.ctx.create_enum_attribute(uwtable_kind, 1);
        fv.add_attribute(inkwell::attributes::AttributeLoc::Function, uwtable_attr);

        // Create LLVM basic blocks.
        for b in &func.blocks {
            let bb = self.ctx.append_basic_block(fv, &format!("bb{}", b.id));
            self.bbs.insert(b.id, bb);
        }

        // Reset per-function state
        self.frame_base = None;
        self.raii_slots.clear();
        self.raii_reg_to_slot.clear();
        self.raii_cleanup_bb = None;
        self.raii_push_counter = 0;
        self.catch_raii_indices.clear();

        // ── Pre-scan: collect all ScopeGuardPush sites to allocate RAII slots ──
        let mut raii_push_sites: Vec<(MirReg, String)> = Vec::new();
        for b in &func.blocks {
            for instr in &b.instrs {
                if let MirInstr::ScopeGuardPush { reg, release_fn, .. } = instr {
                    if !raii_push_sites.iter().any(|(r, _)| r == reg) {
                        raii_push_sites.push((*reg, release_fn.clone()));
                    }
                }
            }
        }

        // Allocas in the entry block.
        let entry = self.bbs[&func.blocks[0].id];
        self.builder.position_at_end(entry);

        // Allocate RAII flag + value slots in the entry block
        let i1_ty = self.ctx.bool_type();
        for (idx, (reg, release_fn)) in raii_push_sites.iter().enumerate() {
            let flag_ptr = self.builder.build_alloca(i1_ty, &format!("raii_flag_{}", idx)).unwrap();
            self.builder.build_store(flag_ptr, i1_ty.const_int(0, false)).unwrap();
            let val_ptr = self.builder.build_alloca(self.i64_ty, &format!("raii_val_{}", idx)).unwrap();
            self.raii_slots.push(crate::codegen::RaiiSlot {
                flag_ptr,
                val_ptr,
                release_fn_name: release_fn.clone(),
            });
            self.raii_reg_to_slot.insert(*reg, idx);
        }

        let regs_array = self.builder.build_alloca(self.i64_ty.array_type(func.next_reg as u32), "regs_array").unwrap();
        
        for rid in 0..func.next_reg {
            let a = unsafe { self.builder.build_gep(self.i64_ty, regs_array, &[self.i32_ty.const_int(rid as u64, false)], &format!("r{}", rid)).unwrap() };
            self.builder.build_store(a, self.nan.const_undefined()).unwrap();
            self.regs.insert(rid, a);
        }

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
            self.exception_scope_stack.clear();
            for &(scope_id, catch_id) in &b.exception_scopes {
                self.exception_scope_stack.push((scope_id, self.bbs[&catch_id]));
            }
            self.builder.position_at_end(self.bbs[&b.id]);
            if func.name.contains("fibonacci") {
                // println!("Codegen Block {}:", b.id);
                for instr in &b.instrs {
                    // println!("  {:?}", instr);
                }
            }
            for instr in &b.instrs {
                let current_bb = self.builder.get_insert_block().unwrap();
                if current_bb.get_terminator().is_some() {
                    if matches!(instr, MirInstr::TryExit | MirInstr::Resume(_, _)) {
                        self.emit_instr(instr)?;
                    }
                    continue;
                }
                let skip_flush = matches!(instr, 
                    MirInstr::Return(_) | MirInstr::Throw(_) | MirInstr::Branch(..) | 
                    MirInstr::Jump(_) | MirInstr::Suspend(..) | MirInstr::FlushRcDelta |
                    MirInstr::RcInc(_) | MirInstr::RcDec(_)
                );
                if !skip_flush {
                    self.flush_deferred_clears();
                }
                self.emit_instr(instr)?;
            }
            self.flush_deferred_clears(); // Just in case a block doesn't end with a terminator (should be rare/invalid, but good practice)
            // Ensure block has a terminator.
            let current_bb = self.builder.get_insert_block().unwrap();
            if current_bb.get_terminator().is_none() {
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
            self.i64_ty.into(), // poll_fn ptr
            self.i64_ty.into(), // drop_fn ptr
            self.i64_ty.into(), // trace_fn ptr
            self.i64_ty.into(), // state_idx
        ];
        for _ in 0..func.next_reg { struct_fields.push(self.i64_ty.into()); }

        let state_ty = self.ctx.struct_type(&struct_fields, false);
        let size_val = state_ty.size_of().unwrap();

        let poll_fn_name = format!("{}_poll", func.name);
        let poll_fn_ty = self.i64_ty.fn_type(&[self.ptr_ty.into(), self.i64_ty.into()], false);
        let poll_fv = self.module.add_function(&poll_fn_name, poll_fn_ty, None);

        let uwtable_kind = inkwell::attributes::Attribute::get_named_enum_kind_id("uwtable");
        let uwtable_attr = self.ctx.create_enum_attribute(uwtable_kind, 1);
        fv.add_attribute(inkwell::attributes::AttributeLoc::Function, uwtable_attr);
        poll_fv.add_attribute(inkwell::attributes::AttributeLoc::Function, uwtable_attr);

        let wrapper_bb = self.ctx.append_basic_block(fv, "entry");
        self.builder.position_at_end(wrapper_bb);

        let alloc_gen_fn = self.module.get_function("__bs_alloc_generator").unwrap();
        let alloc_call = self.builder.build_call(alloc_gen_fn, &[size_val.into()], "gen_alloc").unwrap();
        let gen_ptr_tagged = alloc_call.try_as_basic_value().basic().unwrap().into_int_value();

        let payload = self.builder.build_and(gen_ptr_tagged, self.i64_ty.const_int(crate::nan_box::PAYLOAD_MASK, false), "payload").unwrap();
        let state_ptr = self.builder.build_int_to_ptr(payload, self.ptr_ty, "state_ptr").unwrap();

        let state_idx_ptr = self.builder.build_struct_gep(state_ty, state_ptr, 3, "state_idx_ptr").unwrap();
        self.builder.build_store(state_idx_ptr, self.i64_ty.const_int(0, false)).unwrap();

        let poll_fn_ptr = poll_fv.as_global_value().as_pointer_value();
        let poll_fn_i64 = self.builder.build_ptr_to_int(poll_fn_ptr, self.i64_ty, "poll_fn_i64").unwrap();
        let poll_slot_ptr = self.builder.build_struct_gep(state_ty, state_ptr, 0, "poll_slot").unwrap();
        self.builder.build_store(poll_slot_ptr, poll_fn_i64).unwrap();

        let drop_fn_slot = self.builder.build_struct_gep(state_ty, state_ptr, 1, "drop_fn_slot").unwrap();
        let null_drop_fn = self.i64_ty.const_int(0, false);
        self.builder.build_store(drop_fn_slot, null_drop_fn).unwrap();

        let trace_fn_slot = self.builder.build_struct_gep(state_ty, state_ptr, 2, "trace_fn_slot").unwrap();
        let null_trace_fn = self.i64_ty.const_int(0, false);
        self.builder.build_store(trace_fn_slot, null_trace_fn).unwrap();

        // ── GENERATE DROP AND TRACE FUNCTIONS ──
        let drop_fn_name = format!("{}_drop", func.name);
        let drop_fn = match self.module.get_function(&drop_fn_name) {
            Some(f) => f,
            None => {
                let saved_bb = self.builder.get_insert_block();
                let fn_ty = self.void_ty.fn_type(&[self.ptr_ty.into()], false);
                let f = self.module.add_function(&drop_fn_name, fn_ty, None);
                let entry = self.ctx.append_basic_block(f, "entry");
                self.builder.position_at_end(entry);
                let arg_ptr = f.get_nth_param(0).unwrap().into_pointer_value();
                let circ_dec_fn = self.funcs["circ_dec_tagged"];
                
                // Drop args and locals
                for i in 0..func.next_reg {
                    let offset = self.i32_ty.const_int((4 + i) as u64, false);
                    let slot_ptr = unsafe {
                        self.builder.build_gep(self.i64_ty, arg_ptr, &[offset], "slot").unwrap()
                    };
                    let val = self.builder.build_load(self.i64_ty, slot_ptr, "val").unwrap().into_int_value();
                    self.builder.build_call(circ_dec_fn, &[val.into()], "").unwrap();
                }
                
                self.builder.build_return(None).unwrap();
                if let Some(bb) = saved_bb {
                    self.builder.position_at_end(bb);
                }
                f
            }
        };

        let drop_fn_ptr_val = drop_fn.as_global_value().as_pointer_value();
        let drop_fn_i64 = self.builder.build_ptr_to_int(drop_fn_ptr_val, self.i64_ty, "drop_fn_i64").unwrap();
        let drop_slot_ptr = self.builder.build_struct_gep(state_ty, state_ptr, 1, "drop_slot").unwrap();
        self.builder.build_store(drop_slot_ptr, drop_fn_i64).unwrap();

        let trace_fn_name = format!("{}_trace", func.name);
        let trace_fn = match self.module.get_function(&trace_fn_name) {
            Some(f) => f,
            None => {
                let saved_bb = self.builder.get_insert_block();
                let fn_ty = self.void_ty.fn_type(&[self.ptr_ty.into(), self.ptr_ty.into()], false);
                let f = self.module.add_function(&trace_fn_name, fn_ty, None);
                let entry = self.ctx.append_basic_block(f, "entry");
                self.builder.position_at_end(entry);
                let arg_ptr = f.get_nth_param(0).unwrap().into_pointer_value();
                let visitor_ptr = f.get_nth_param(1).unwrap().into_pointer_value();
                
                for i in 0..func.next_reg {
                    let offset = self.i32_ty.const_int((4 + i) as u64, false);
                    let slot_ptr = unsafe {
                        self.builder.build_gep(self.i64_ty, arg_ptr, &[offset], "slot").unwrap()
                    };
                    let val = self.builder.build_load(self.i64_ty, slot_ptr, "val").unwrap().into_int_value();
                    self.emit_trace_visitor_if_heap_object(val, visitor_ptr);
                }
                
                self.builder.build_return(None).unwrap();
                if let Some(bb) = saved_bb {
                    self.builder.position_at_end(bb);
                }
                f
            }
        };

        let trace_fn_ptr_val = trace_fn.as_global_value().as_pointer_value();
        let trace_fn_i64 = self.builder.build_ptr_to_int(trace_fn_ptr_val, self.i64_ty, "trace_fn_i64").unwrap();
        let trace_slot_ptr = self.builder.build_struct_gep(state_ty, state_ptr, 2, "trace_slot").unwrap();
        self.builder.build_store(trace_slot_ptr, trace_fn_i64).unwrap();

        for (i, _) in func.params.iter().enumerate() {
            let arg_val = fv.get_nth_param(i as u32).unwrap().into_int_value();
            let arg_slot = self.builder.build_struct_gep(state_ty, state_ptr, 4 + i as u32, "arg_slot").unwrap();
            self.builder.build_store(arg_slot, arg_val).unwrap();
        }
        
        let undef_val = self.i64_ty.const_int(crate::nan_box::TAG_UNDEFINED, false);
        for i in func.params.len() as u32..func.next_reg {
            let local_slot = self.builder.build_struct_gep(state_ty, state_ptr, 4 + i, "local_slot").unwrap();
            self.builder.build_store(local_slot, undef_val).unwrap();
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

        self.frame_base = None;
        let uses_raii = func.blocks.iter().any(|b| {
            b.instrs.iter().any(|i| matches!(
                i,
                MirInstr::ScopeGuardPush { .. } | MirInstr::ScopeGuardCancel { .. } | MirInstr::ScopeGuardFlushTo { .. }
            ))
        });

        let poll_entry = self.ctx.append_basic_block(poll_fv, "entry");
        self.builder.position_at_end(poll_entry);

        if uses_raii {
            let scope_guard_get_len_fn = self.module.get_function("__bs_scope_guard_get_len").unwrap_or_else(|| {
                let ty = self.i32_ty.fn_type(&[], false);
                self.module.add_function("__bs_scope_guard_get_len", ty, None)
            });
            let call = self.builder.build_call(scope_guard_get_len_fn, &[], "frame_base").unwrap();
            self.frame_base = Some(call.try_as_basic_value().basic().unwrap().into_int_value());
        }

        let state_arg = poll_fv.get_nth_param(0).unwrap().into_pointer_value();
        let sent_val_arg = poll_fv.get_nth_param(1).unwrap().into_int_value();
        self.gen_state_ptr = Some(state_arg);
        self.gen_sent_val = Some(sent_val_arg);

        for (i, (reg, _)) in func.params.iter().enumerate() {
            let arg_slot = self.builder.build_struct_gep(state_ty, state_arg, 4 + i as u32, &format!("r{}", reg)).unwrap();
            self.regs.insert(*reg, arg_slot);
        }

        for rid in 0..func.next_reg {
            if !self.regs.contains_key(&rid) {
                let local_slot = self.builder.build_struct_gep(state_ty, state_arg, 4 + rid, &format!("r{}", rid)).unwrap();
                self.regs.insert(rid, local_slot);
            }
        }

        // Shadow stack removed.

        for b in &func.blocks {
            let bb = self.ctx.append_basic_block(poll_fv, &format!("bb{}", b.id));
            self.bbs.insert(b.id, bb);
        }

        let done_bb = self.ctx.append_basic_block(poll_fv, "done");

        for i in 0..func.num_yield_points {
            let rbb = self.ctx.append_basic_block(poll_fv, &format!("resume_{}", i));
            self.resume_blocks.insert(i, rbb);
        }

        let state_idx_ptr = self.builder.build_struct_gep(state_ty, state_arg, 3, "state_idx_ptr").unwrap();
        let state_val = self.builder.build_load(self.i64_ty, state_idx_ptr, "state_val").unwrap().into_int_value();

        let mut switch_cases = Vec::new();
        switch_cases.push((self.i64_ty.const_int(0, false), self.bbs[&func.blocks[0].id]));
        for i in 0..func.num_yield_points {
            switch_cases.push((self.i64_ty.const_int((i + 1) as u64, false), self.resume_blocks[&i]));
        }

        self.builder.build_switch(state_val, done_bb, &switch_cases).unwrap();

        self.builder.position_at_end(done_bb);
        self.builder.build_return(Some(&self.nan.const_undefined())).unwrap();

        for b in &func.blocks {
            self.exception_scope_stack.clear();
            for &(scope_id, catch_id) in &b.exception_scopes {
                self.exception_scope_stack.push((scope_id, self.bbs[&catch_id]));
            }
            self.builder.position_at_end(self.bbs[&b.id]);
            if func.name.contains("fibonacci") {
                // println!("Codegen Block {}:", b.id);
                for instr in &b.instrs {
                    // println!("  {:?}", instr);
                }
            }
            for instr in &b.instrs {
                let current_bb = self.builder.get_insert_block().unwrap();
                if current_bb.get_terminator().is_some() {
                    if matches!(instr, MirInstr::TryExit | MirInstr::Resume(_, _)) {
                        self.emit_instr(instr)?;
                    }
                    continue;
                }
                let is_term = matches!(instr, MirInstr::Return(_) | MirInstr::Throw(_) | MirInstr::Branch(..) | MirInstr::Jump(_) | MirInstr::Suspend(..));
                if !is_term {
                    self.flush_deferred_clears();
                }
                self.emit_instr(instr)?;
            }
            self.flush_deferred_clears();
            let current_bb = self.builder.get_insert_block().unwrap();
            if current_bb.get_terminator().is_none() {
                if let Some(state_ptr) = self.gen_state_ptr {
                    let state_idx_ptr = self.builder.build_struct_gep(state_ty, state_ptr, 3, "state_idx_ptr").unwrap();
                    self.builder.build_store(state_idx_ptr, self.i64_ty.const_all_ones()).unwrap();
                }
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

        let uwtable_kind = inkwell::attributes::Attribute::get_named_enum_kind_id("uwtable");
        let uwtable_attr = self.ctx.create_enum_attribute(uwtable_kind, 1);
        main_fn.add_attribute(inkwell::attributes::AttributeLoc::Function, uwtable_attr);

        self.regs.clear();
        self.bbs.clear();

        for b in &body.blocks {
            let bb = self.ctx.append_basic_block(main_fn, &format!("bb{}", b.id));
            self.bbs.insert(b.id, bb);
        }

        self.frame_base = None;
        self.raii_slots.clear();
        self.raii_reg_to_slot.clear();
        self.raii_cleanup_bb = None;
        self.raii_push_counter = 0;
        self.catch_raii_indices.clear();

        // Pre-scan for RAII slots
        let mut raii_push_sites: Vec<(MirReg, String)> = Vec::new();
        for b in &body.blocks {
            for instr in &b.instrs {
                if let MirInstr::ScopeGuardPush { reg, release_fn, .. } = instr {
                    if !raii_push_sites.iter().any(|(r, _)| r == reg) {
                        raii_push_sites.push((*reg, release_fn.clone()));
                    }
                }
            }
        }

        let entry = self.bbs[&body.blocks[0].id];
        self.builder.position_at_end(entry);

        // Initialize cycle collector
        let cycle_init_fn = self.module.get_function("__bs_cycle_collector_init").unwrap();
        self.builder.build_call(cycle_init_fn, &[], "init_cycle_collector").unwrap();

        let set_verify_fn = self.module.get_function("__bs_set_verify_memory").unwrap();
        let verify_arg = self.i8_ty.const_int(if self.verify_memory { 1 } else { 0 }, false);
        self.builder.build_call(set_verify_fn, &[verify_arg.into()], "set_verify_memory").unwrap();

        // Allocate RAII flag + value slots
        let i1_ty = self.ctx.bool_type();
        for (idx, (reg, release_fn)) in raii_push_sites.iter().enumerate() {
            let flag_ptr = self.builder.build_alloca(i1_ty, &format!("raii_flag_{}", idx)).unwrap();
            self.builder.build_store(flag_ptr, i1_ty.const_int(0, false)).unwrap();
            let val_ptr = self.builder.build_alloca(self.i64_ty, &format!("raii_val_{}", idx)).unwrap();
            self.raii_slots.push(crate::codegen::RaiiSlot {
                flag_ptr,
                val_ptr,
                release_fn_name: release_fn.clone(),
            });
            self.raii_reg_to_slot.insert(*reg, idx);
        }

        let array_ty = self.i64_ty.array_type(body.next_reg as u32);
        let regs_array = self.builder.build_alloca(array_ty, "regs_array").unwrap();

        for rid in 0..body.next_reg {
            let a = unsafe { self.builder.build_gep(array_ty, regs_array, &[self.i32_ty.const_int(0, false), self.i32_ty.const_int(rid as u64, false)], &format!("r{}", rid)).unwrap() };
            self.builder.build_store(a, self.nan.const_undefined()).unwrap();
            self.regs.insert(rid, a);
        }

        for b in &body.blocks {
            self.builder.position_at_end(self.bbs[&b.id]);
            for instr in &b.instrs {
                let current_bb = self.builder.get_insert_block().unwrap();
                if current_bb.get_terminator().is_some() {
                    // TryExit only modifies compile-time state, so it's safe to run even if the block is terminated
                    if matches!(instr, MirInstr::TryExit | MirInstr::Resume(_, _)) {
                        self.emit_instr(instr)?;
                    }
                    continue;
                }
                // Skip Return instructions — main always returns i32 0.
                if matches!(instr, MirInstr::Return(_)) {
                    continue;
                }
                let is_term = matches!(instr, MirInstr::Return(_) | MirInstr::Throw(_) | MirInstr::Branch(..) | MirInstr::Jump(_) | MirInstr::Suspend(..));
                if !is_term {
                    self.flush_deferred_clears();
                }
                self.emit_instr(instr)?;
            }
            self.flush_deferred_clears();
            let current_bb = self.builder.get_insert_block().unwrap();
            if current_bb.get_terminator().is_none() {
                let drain_fn = self.module.get_function("__bs_drain_microtasks").unwrap();
                self.builder.build_call(drain_fn, &[], "drain").unwrap();

                let drain_finalizers_fn = self.module.get_function("__bs_drain_finalizers");
                if let Some(df) = drain_finalizers_fn {
                    self.builder.build_call(df, &[], "drain_finalizers").unwrap();
                }

                if self.verify_memory {
                    // Drop all JS global variables (they are stored in LLVM i64 globals)
                    let circ_dec_fn = self.module.get_function("circ_dec_tagged").unwrap();
                    let i64_ty = self.i64_ty;
                    let mut globals_to_dec = Vec::new();
                    for global in self.module.get_globals() {
                        if global.get_value_type().is_int_type() && global.get_value_type().into_int_type() == i64_ty {
                            globals_to_dec.push(global.as_pointer_value());
                        }
                    }
                    for global_ptr in globals_to_dec {
                        let val = self.builder.build_load(i64_ty, global_ptr, "global_val").unwrap().into_int_value();
                        self.builder.build_call(circ_dec_fn, &[val.into()], "dec_global").unwrap();
                        self.builder.build_store(global_ptr, self.nan.const_undefined()).unwrap();
                    }

                    if let Some(check_leaks) = self.module.get_function("__bs_verify_check_leaks") {
                        self.builder.build_call(check_leaks, &[], "check_leaks").unwrap();
                    }
                }

                self.builder
                    .build_return(Some(&self.i32_ty.const_int(0, false)))
                    .unwrap();
            }
        }
        Ok(())
    }
}
