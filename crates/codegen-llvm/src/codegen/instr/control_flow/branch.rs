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
    pub(in crate::codegen::instr) fn emit_instr_branch(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::Branch(cond, t, f) => {
                let cv_val = self.val(cond)?;
    let cv = self.nan.is_truthy(&self.builder, cv_val);
    let tbb = self.bbs[t];
    let fbb = self.bbs[f];
    self.builder.build_conditional_branch(cv, tbb, fbb).unwrap();
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
