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
    pub(in crate::codegen::instr) fn emit_instr_unary_neg(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::Neg(d, v) => {
                let vv = self.val(v)?;
    let fv = self.nan.unbox_number(&self.builder, vv);
    let r = self.builder.build_float_neg(fv, "neg").unwrap();
    self.store(*d, self.nan.box_number(&self.builder, r));
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
