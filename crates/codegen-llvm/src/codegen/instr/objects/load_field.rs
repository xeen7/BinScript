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
    pub(in crate::codegen::instr) fn emit_instr_load_field(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::LoadField(dest, obj_reg, index) => {
                let obj_val = self.val(&MirOperand::Reg(*obj_reg))?;
    let payload = self.builder.build_and(obj_val, self.i64_ty.const_int(crate::nan_box::PAYLOAD_MASK, false), "payload").unwrap();
    let obj_ptr = self.builder.build_int_to_ptr(payload, self.ptr_ty, "obj_ptr").unwrap();

    let offset = self.i32_ty.const_int((2 + index) as u64, false);
    let field_ptr = unsafe {
        self.builder.build_gep(self.i64_ty, obj_ptr, &[offset], "field_ptr").unwrap()
    };
    
    if self.verify_memory {
        let verify_fn = self.module.get_function("__verify_load").unwrap();
        self.builder.build_call(verify_fn, &[obj_ptr.into()], "verify_load_call").unwrap();
    }
    let loaded_val = self.builder.build_load(self.i64_ty, field_ptr, "loaded").unwrap().into_int_value();
    self.store(*dest, loaded_val);
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
