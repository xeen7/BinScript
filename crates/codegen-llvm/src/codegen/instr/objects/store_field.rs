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
    pub(in crate::codegen::instr) fn emit_instr_store_field(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::StoreField(obj_reg, index, val_operand) => {
                let obj_val = self.val(&MirOperand::Reg(*obj_reg))?;
    let payload = self.builder.build_and(obj_val, self.i64_ty.const_int(crate::nan_box::PAYLOAD_MASK, false), "payload").unwrap();
    let obj_ptr = self.builder.build_int_to_ptr(payload, self.ptr_ty, "obj_ptr").unwrap();

    let offset = self.i32_ty.const_int((1 + index) as u64, false);
    let field_ptr = unsafe {
        self.builder.build_gep(self.i64_ty, obj_ptr, &[offset], "field_ptr").unwrap()
    };
    let val_to_store = self.val(val_operand)?;
    self.builder.build_store(field_ptr, val_to_store).unwrap();
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
