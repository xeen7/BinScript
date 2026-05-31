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
    pub(in crate::codegen::instr) fn emit_instr_load_global(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::LoadGlobal(dest, name) => {
                let global = self.module.get_global(name).unwrap_or_else(|| {
        let g = self.module.add_global(self.i64_ty, None, name);
        g.set_initializer(&self.nan.const_undefined());
        g
    });
    let loaded = self.builder.build_load(self.i64_ty, global.as_pointer_value(), "load_global").unwrap().into_int_value();
    self.store(*dest, loaded);
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
