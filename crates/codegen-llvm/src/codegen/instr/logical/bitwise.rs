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
    pub(in crate::codegen::instr) fn emit_instr_bitwise(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::BitAnd(d, l, r) => self.emit_bitwise_i32(*d, l, r, "and")?,
            MirInstr::BitOr(d, l, r)  => self.emit_bitwise_i32(*d, l, r, "or")?,
            MirInstr::BitXor(d, l, r) => self.emit_bitwise_i32(*d, l, r, "xor")?,
            MirInstr::Shl(d, l, r)    => self.emit_bitwise_i32(*d, l, r, "shl")?,
            MirInstr::Shr(d, l, r)    => self.emit_bitwise_i32(*d, l, r, "ashr")?,
            MirInstr::UShr(d, l, r)   => self.emit_bitwise_u32(*d, l, r)?,
            MirInstr::BitNot(d, v) => {
                let vv = self.val(v)?;
                let fv = self.nan.unbox_number(&self.builder, vv);
                let iv = self.builder.build_float_to_signed_int(fv, self.i32_ty, "toi32").unwrap();
                let notted = self.builder.build_not(iv, "bitnot").unwrap();
                let f64_ty = self.ctx.f64_type();
                let back = self.builder.build_signed_int_to_float(notted, f64_ty, "tof64").unwrap();
                self.store(*d, self.nan.box_number(&self.builder, back));
            }
            _ => unreachable!()
        }
        Ok(())
    }
}
