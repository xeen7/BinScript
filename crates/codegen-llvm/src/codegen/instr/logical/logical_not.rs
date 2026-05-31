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
    pub(in crate::codegen::instr) fn emit_instr_logical_not(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::Not(d, v) => {
                let vv = self.val(v)?;
    let truthy = self.nan.is_truthy(&self.builder, vv);
    let neg = self.builder.build_not(truthy, "not").unwrap();
    self.store(*d, self.nan.box_bool(&self.builder, neg));
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
