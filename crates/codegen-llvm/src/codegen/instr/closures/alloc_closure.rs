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
    pub(in crate::codegen::instr) fn emit_instr_alloc_closure(&mut self, instr: &MirInstr) -> CompileResult<()> {
        let (dest, func_id, captures, is_owned) = match instr {
            MirInstr::AllocClosure(d, f, c) | MirInstr::AllocSharedClosure(d, f, c) => (d, f, c, false),
            MirInstr::AllocOwnedClosure(d, f, c) => (d, f, c, true),
            _ => return Ok(()),
        };

        let func_name = self.func_id_to_name.get(func_id).ok_or_else(|| {
            CompileError::Codegen {
                message: format!("unknown func_id {}", func_id),
            }
        })?;
        let fv = self.funcs.get(func_name).copied().ok_or_else(|| {
            CompileError::Codegen {
                message: format!("unknown fn {}", func_name),
            }
        })?;

        // Calculate allocation size: 8 * (3 + captures.len()) bytes
        let size_in_bytes = 8 * (3 + captures.len());
        let alloc_fn_name = if is_owned {
            "__bs_alloc_owned_closure"
        } else {
            "__bs_alloc_closure"
        };
        let alloc_fn = self.funcs[alloc_fn_name];
        let size_val = self.i64_ty.const_int(size_in_bytes as u64, false);

        // Call __bs_alloc_closure(size)
        let closure_val = self.builder.build_call(alloc_fn, &[size_val.into()], "alloc_closure").unwrap()
            .try_as_basic_value()
            .basic()
            .unwrap()
            .into_int_value();

        let mask = self.i64_ty.const_int(0x0000_FFFF_FFFF_FFFF, false);
        let raw_ptr_i64 = self.builder.build_and(closure_val, mask, "unbox_ptr").unwrap();
        let closure_ptr = self.builder.build_int_to_ptr(raw_ptr_i64, self.ptr_ty, "ptr").unwrap();

        // Store function pointer at offset 0
        let fn_ptr = fv.as_global_value().as_pointer_value();
        let offset0 = self.i32_ty.const_int(0, false);
        let fn_slot = unsafe {
            self.builder.build_gep(self.ptr_ty, closure_ptr, &[offset0], "fn_slot").unwrap()
        };
        self.builder.build_store(fn_slot, fn_ptr).unwrap();

        // Generate or get drop function pointer
        let drop_fn_name = format!("__bs_closure_drop_{}", func_id);
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
                    for i in 0..captures.len() {
                        let offset = self.i32_ty.const_int((3 + i) as u64, false);
                        let capture_slot = unsafe {
                            self.builder.build_gep(self.i64_ty, arg_ptr, &[offset], "capture_slot").unwrap()
                        };
                        let cap_val = self.builder.build_load(self.i64_ty, capture_slot, "cap_val").unwrap().into_int_value();
                        self.builder.build_call(circ_dec_fn, &[cap_val.into()], "").unwrap();
                    }
                    
                    self.builder.build_return(None).unwrap();
                    
                    if let Some(bb) = saved_bb {
                        self.builder.position_at_end(bb);
                    }
                    f
                }
        };
        let drop_fn_ptr = drop_fn.as_global_value().as_pointer_value();

        // Store drop function pointer at offset 1
        let offset1 = self.i32_ty.const_int(1, false);
        let drop_slot = unsafe {
            self.builder.build_gep(self.ptr_ty, closure_ptr, &[offset1], "drop_slot").unwrap()
        };
        self.builder.build_store(drop_slot, drop_fn_ptr).unwrap();

    // Generate or get trace function
    let trace_fn_name = format!("__bs_closure_trace_{}", func_id);
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
            
            for i in 0..captures.len() {
                let offset = self.i32_ty.const_int((3 + i) as u64, false);
                let capture_slot = unsafe {
                    self.builder.build_gep(self.i64_ty, arg_ptr, &[offset], "capture_slot").unwrap()
                };
                let cap_val = self.builder.build_load(self.i64_ty, capture_slot, "cap_val").unwrap().into_int_value();
                self.emit_trace_visitor_if_heap_object(cap_val, visitor_ptr);
            }
            
            self.builder.build_return(None).unwrap();
            
            if let Some(bb) = saved_bb {
                self.builder.position_at_end(bb);
            }
            f
        }
    };

    // Store trace function pointer at offset 2
    let trace_fn_ptr = trace_fn.as_global_value().as_pointer_value();
    let offset2 = self.i32_ty.const_int(2, false);
    let trace_slot = unsafe {
        self.builder.build_gep(self.ptr_ty, closure_ptr, &[offset2], "trace_slot").unwrap()
    };
    self.builder.build_store(trace_slot, trace_fn_ptr).unwrap();

    // Store each capture at offset 3 + i
    let circ_inc_fn = self.funcs["circ_inc_tagged"];
    for (i, cap) in captures.iter().enumerate() {
        let val_to_store = self.val(cap)?;
        let offset = self.i32_ty.const_int((3 + i) as u64, false);
        let capture_slot = unsafe {
            self.builder.build_gep(self.i64_ty, closure_ptr, &[offset], "capture_slot").unwrap()
        };
        self.builder.build_store(capture_slot, val_to_store).unwrap();
    }

    // Store tagged pointer in dest
    self.store(*dest, closure_val);
        Ok(())
    }
}
