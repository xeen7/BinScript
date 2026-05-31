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
    pub(in crate::codegen::instr) fn emit_instr_modulo(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::Mod(d, l, r) => {
                self.emit_arith_f64(*d, l, r, "frem")?
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
