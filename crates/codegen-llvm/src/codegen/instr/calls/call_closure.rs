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
    pub(in crate::codegen::instr) fn emit_instr_call_closure(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::CallClosure(dest, callee_reg, args) => {
                let callee_val = self.val(&MirOperand::Reg(*callee_reg))?;
    let payload = self.builder.build_and(callee_val, self.i64_ty.const_int(crate::nan_box::PAYLOAD_MASK, false), "payload").unwrap();
    let closure_ptr = self.builder.build_int_to_ptr(payload, self.ptr_ty, "closure_ptr").unwrap();

    // Load function pointer from offset 0
    let offset0 = self.i32_ty.const_int(0, false);
    let fn_slot = unsafe {
        self.builder.build_gep(self.ptr_ty, closure_ptr, &[offset0], "fn_slot").unwrap()
    };
    let fn_ptr = self.builder.build_load(self.ptr_ty, fn_slot, "fn_ptr").unwrap().into_pointer_value();

    let mut param_types = Vec::new();
    for _ in 0..args.len() {
        param_types.push(self.i64_ty.into());
    }
    let fn_ty = self.i64_ty.fn_type(&param_types, false);

    let av: Vec<BasicMetadataValueEnum<'ctx>> = args
        .iter()
        .map(|a| self.val(a).map(|v| v.into()))
        .collect::<CompileResult<_>>()?;

    let rv = self.builder.build_indirect_call(fn_ty, fn_ptr, &av, "closure_call").unwrap();
    let v = rv
        .try_as_basic_value()
        .basic()
        .map(|bv| bv.into_int_value())
        .unwrap_or_else(|| self.nan.const_undefined());
    self.store(*dest, v);
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
