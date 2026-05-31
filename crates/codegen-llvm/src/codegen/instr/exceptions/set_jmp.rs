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
    pub(in crate::codegen::instr) fn emit_instr_set_jmp(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::SetJmp(dest_reg, jmp_buf_reg) => {
                let jmp_buf_int = self.val(&MirOperand::Reg(*jmp_buf_reg))?;
    let jmp_buf_ptr = self.builder.build_int_to_ptr(jmp_buf_int, self.ptr_ty, "jmp_buf_ptr").unwrap();
    let setjmp_fn = self.module.get_function("_setjmp").unwrap();
    
    let res = self.builder.build_call(setjmp_fn, &[jmp_buf_ptr.into()], "setjmp_res").unwrap()
        .try_as_basic_value()
        .basic()
        .unwrap()
        .into_int_value();
    
    // Compare setjmp result to 0
    let zero = self.i32_ty.const_int(0, false);
    let is_nonzero = self.builder.build_int_compare(IntPredicate::NE, res, zero, "is_nonzero").unwrap();
    
    // Box as Boolean: true if nonzero (longjmp return), false if zero (direct return)
    let boxed_bool = self.nan.box_bool(&self.builder, is_nonzero);
    self.store(*dest_reg, boxed_bool);
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
