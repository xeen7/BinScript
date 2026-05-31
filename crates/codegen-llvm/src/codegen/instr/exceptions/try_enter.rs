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
    pub(in crate::codegen::instr) fn emit_instr_try_enter(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::TryEnter(jmp_buf_reg) => {
                let jmp_buf_ty = self.i8_ty.array_type(256);
    let jmp_buf_alloca = self.builder.build_alloca(jmp_buf_ty, "jmp_buf").unwrap();
    let jmp_buf_int = self.builder.build_ptr_to_int(jmp_buf_alloca, self.i64_ty, "jmp_buf_int").unwrap();
    self.store(*jmp_buf_reg, jmp_buf_int);

    let try_enter_fn = self.module.get_function("__bs_try_enter").unwrap();
    let ptr_val = self.builder.build_int_to_ptr(jmp_buf_int, self.ptr_ty, "jmp_buf_ptr").unwrap();
    self.builder.build_call(try_enter_fn, &[ptr_val.into()], "").unwrap();
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
