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
    pub(in crate::codegen::instr) fn emit_instr_store_prop(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::StoreProp(obj_reg, prop_name, val_operand) => {
                let prop_set_fn = self.module.get_function("__bs_prop_set").unwrap();
    let obj_val = self.val(&MirOperand::Reg(*obj_reg))?;
    let global_str = self.builder.build_global_string_ptr(prop_name, "prop_str").unwrap();
    let prop_ptr = global_str.as_pointer_value();
    let prop_len = self.i32_ty.const_int(prop_name.len() as u64, false);
    let val = self.val(val_operand)?;
    
    self.builder.build_call(prop_set_fn, &[obj_val.into(), prop_ptr.into(), prop_len.into(), val.into()], "prop_set").unwrap();
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
