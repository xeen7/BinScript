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
    pub(in crate::codegen::instr) fn emit_instr_store_global(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::StoreGlobal(name, val_operand) => {
                let global = self.module.get_global(name).unwrap_or_else(|| {
                    let g = self.module.add_global(self.i64_ty, None, name);
                    g.set_initializer(&self.nan.const_undefined());
                    g
                });
                let val = self.val(val_operand)?;

                // Load old value and decrement its refcount
                let old_val = self.builder.build_load(self.i64_ty, global.as_pointer_value(), "old_val").unwrap().into_int_value();
                let circ_dec_fn = self.module.get_function("circ_dec_tagged").unwrap();
                self.builder.build_call(circ_dec_fn, &[old_val.into()], "dec_old_global").unwrap();

                // Increment refcount for new value
                let circ_inc_fn = self.module.get_function("circ_inc_tagged").unwrap();
                self.builder.build_call(circ_inc_fn, &[val.into()], "inc_new_global").unwrap();

                // Store new value
                self.builder.build_store(global.as_pointer_value(), val).unwrap();
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
