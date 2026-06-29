#![allow(unused_imports)]
#![allow(unused_unsafe)]
use mir::types::*;
use diagnostics::CompileResult;
use crate::codegen::LlvmCodegen;

impl<'ctx> LlvmCodegen<'ctx> {
    pub(in crate::codegen::instr) fn emit_instr_extract_exception(&mut self, instr: &MirInstr) -> CompileResult<()> {
        if let MirInstr::ExtractException { dest, lp_reg } = instr {
            // Get the struct pointer from lp_reg
            let lp_ptr_i64 = self.val(&MirOperand::Reg(*lp_reg))?;
            let lp_ptr = self.builder.build_int_to_ptr(lp_ptr_i64, self.ptr_ty, "lp_ptr").unwrap();
            
            // The type is { ptr, i32 }
            let lp_ty = self.ctx.struct_type(&[
                self.ptr_ty.into(),
                self.i32_ty.into(),
            ], false);
            
            // Extract the exception object pointer (field 0)
            let exn_obj_ptr_ptr = self.builder.build_struct_gep(lp_ty, lp_ptr, 0, "exn_obj_ptr_ptr").unwrap();
            let exn_obj_ptr = self.builder.build_load(self.ptr_ty, exn_obj_ptr_ptr, "exn_obj_ptr").unwrap();
            
            let ext_fn = self.module.get_function("__bs_get_exception_value").unwrap_or_else(|| {
                let ty = self.i64_ty.fn_type(&[self.ptr_ty.into()], false);
                self.module.add_function("__bs_get_exception_value", ty, None)
            });
            
            let val = self.builder.build_call(ext_fn, &[exn_obj_ptr.into()], "exn_val").unwrap().try_as_basic_value().basic().unwrap().into_int_value();
            self.store(*dest, val);

            let free_fn = self.module.get_function("__bs_free_exception").unwrap_or_else(|| {
                let ty = self.ctx.void_type().fn_type(&[self.ptr_ty.into()], false);
                self.module.add_function("__bs_free_exception", ty, None)
            });
            self.builder.build_call(free_fn, &[exn_obj_ptr.into()], "").unwrap();
        }
        Ok(())
    }
}
