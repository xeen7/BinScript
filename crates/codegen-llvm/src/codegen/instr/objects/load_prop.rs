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
    pub(in crate::codegen::instr) fn emit_instr_load_prop(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::LoadProp(dest, obj_reg, prop_name) => {
                let prop_get_fn = self.module.get_function("__bs_prop_get").unwrap();
    let obj_val = self.val(&MirOperand::Reg(*obj_reg))?;
    let global_str = self.builder.build_global_string_ptr(prop_name, "prop_str").unwrap();
    let prop_ptr = global_str.as_pointer_value();
    let prop_len = self.i32_ty.const_int(prop_name.len() as u64, false);
    
    let rv = self.builder.build_call(prop_get_fn, &[obj_val.into(), prop_ptr.into(), prop_len.into()], "prop_get").unwrap();
    let v = rv.try_as_basic_value().basic().unwrap().into_int_value();
    self.store(*dest, v);
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
