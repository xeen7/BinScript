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
        match instr {
            MirInstr::AllocClosure(dest, func_id, captures) => {
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

    // Calculate allocation size: 8 * (2 + captures.len()) bytes
    let size_in_bytes = 8 * (2 + captures.len());
    let alloc_fn = self.funcs["__bs_alloc_closure"];
    let size_val = self.i64_ty.const_int(size_in_bytes as u64, false);

    // Call __bs_alloc_closure(size)
    let closure_val = self.builder.build_call(alloc_fn, &[size_val.into()], "alloc_closure").unwrap()
        .try_as_basic_value()
        .basic()
        .unwrap()
        .into_int_value();

    // Extract raw closure pointer
    let payload = self.builder.build_and(closure_val, self.i64_ty.const_int(crate::nan_box::PAYLOAD_MASK, false), "payload").unwrap();
    let closure_ptr = self.builder.build_int_to_ptr(payload, self.ptr_ty, "closure_ptr").unwrap();

    // Store function pointer at offset 0
    let fn_ptr = fv.as_global_value().as_pointer_value();
    let offset0 = self.i32_ty.const_int(0, false);
    let fn_slot = unsafe {
        self.builder.build_gep(self.ptr_ty, closure_ptr, &[offset0], "fn_slot").unwrap()
    };
    self.builder.build_store(fn_slot, fn_ptr).unwrap();

    // Store undefined at offset 1
    let offset1 = self.i32_ty.const_int(1, false);
    let unused_slot = unsafe {
        self.builder.build_gep(self.i64_ty, closure_ptr, &[offset1], "unused_slot").unwrap()
    };
    self.builder.build_store(unused_slot, self.nan.const_undefined()).unwrap();

    // Store each capture at offset 2 + i
    for (i, cap) in captures.iter().enumerate() {
        let val_to_store = self.val(cap)?;
        let offset = self.i32_ty.const_int((2 + i) as u64, false);
        let capture_slot = unsafe {
            self.builder.build_gep(self.i64_ty, closure_ptr, &[offset], "capture_slot").unwrap()
        };
        self.builder.build_store(capture_slot, val_to_store).unwrap();
    }

    // Store tagged pointer in dest
    self.store(*dest, closure_val);
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
