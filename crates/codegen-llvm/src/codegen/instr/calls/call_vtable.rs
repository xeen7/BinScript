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
    pub(in crate::codegen::instr) fn emit_instr_call_vtable(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::CallVTable(dest, obj_reg, method_index, args) => {
                let obj_val = self.val(&MirOperand::Reg(*obj_reg))?;
    let payload = self.builder.build_and(obj_val, self.i64_ty.const_int(crate::nan_box::PAYLOAD_MASK, false), "payload").unwrap();
    let obj_ptr = self.builder.build_int_to_ptr(payload, self.ptr_ty, "obj_ptr").unwrap();

    let vtable_ptr = self.builder.build_load(self.ptr_ty, obj_ptr, "vtable_ptr").unwrap().into_pointer_value();

    let method_offset = self.i32_ty.const_int((5 + method_index) as u64, false);
    let method_fn_ptr_ptr = unsafe {
        self.builder.build_gep(self.ptr_ty, vtable_ptr, &[method_offset], "method_fn_ptr_ptr").unwrap()
    };
    let method_fn_ptr = self.builder.build_load(self.ptr_ty, method_fn_ptr_ptr, "method_fn_ptr").unwrap().into_pointer_value();

    let mut param_types = Vec::new();
    for _ in 0..args.len() {
        param_types.push(self.i64_ty.into());
    }
    let fn_ty = self.i64_ty.fn_type(&param_types, false);

    let av: Vec<BasicMetadataValueEnum<'ctx>> = args
        .iter()
        .map(|a| self.val(a).map(|v| v.into()))
        .collect::<CompileResult<_>>()?;
    let rv = self.builder.build_indirect_call(fn_ty, method_fn_ptr, &av, "vcall").unwrap();
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
