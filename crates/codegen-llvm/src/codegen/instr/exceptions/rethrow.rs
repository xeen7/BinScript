#![allow(unused_imports)]
#![allow(unused_unsafe)]
use mir::types::*;
use diagnostics::CompileResult;
use crate::codegen::LlvmCodegen;

impl<'ctx> LlvmCodegen<'ctx> {
    pub(in crate::codegen::instr) fn emit_instr_rethrow(&mut self, instr: &MirInstr) -> CompileResult<()> {
        if let MirInstr::Rethrow(lp_reg) = instr {
            // Load the struct pointer from lp_reg
            let lp_ptr_i64 = self.val(&MirOperand::Reg(*lp_reg))?;
            let lp_ptr = self.builder.build_int_to_ptr(lp_ptr_i64, self.ptr_ty, "lp_ptr").unwrap();
            
            // The type is { ptr, i32 }
            let lp_ty = self.ctx.struct_type(&[
                self.ptr_ty.into(),
                self.i32_ty.into(),
            ], false);
            
            let lp_val = self.builder.build_load(lp_ty, lp_ptr, "lp_val").unwrap();
            
            // Resume unwinding
            self.builder.build_resume(lp_val).unwrap();
        }
        Ok(())
    }
}
