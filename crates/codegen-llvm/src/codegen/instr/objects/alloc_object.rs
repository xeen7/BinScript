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
    pub(in crate::codegen::instr) fn emit_instr_alloc_object(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::Alloc(dest, class_name) => {
                let fields_count = self.get_all_fields_count(class_name);
    let size_in_bytes = 8 * (1 + fields_count);
    let vtable_g = self.vtables.get(class_name).ok_or_else(|| {
        CompileError::Codegen {
            message: format!("Vtable not found for class {}", class_name),
        }
    })?;
    let vtable_ptr = vtable_g.as_pointer_value();

    let alloc_fn = self.funcs["__bs_alloc"];
    let size_val = self.i64_ty.const_int(size_in_bytes as u64, false);

    let obj_val = self.builder.build_call(alloc_fn, &[vtable_ptr.into(), size_val.into()], "alloc").unwrap()
        .try_as_basic_value()
        .basic()
        .unwrap()
        .into_int_value();

    self.store(*dest, obj_val);
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
