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
    pub(in crate::codegen::instr) fn emit_instr_call_direct(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::CallDirect(d, name, args) => {
                let fn_val = self.funcs.get(name).copied().ok_or_else(|| {
        CompileError::Codegen { message: format!("unknown fn {}", name) }
    })?;
    let mut av: Vec<BasicMetadataValueEnum<'ctx>> = args
        .iter()
        .map(|a| self.val(a).map(|v| v.into()))
        .collect::<CompileResult<_>>()?;
    let expected_params = fn_val.count_params() as usize;
    if expected_params == av.len() + 1 {
        av.insert(0, self.nan.const_undefined().into());
    }
    let rv = self.builder.build_call(fn_val, &av, "call").unwrap();
    let v = rv
        .try_as_basic_value()
        .basic()
        .map(|bv| bv.into_int_value())
        .unwrap_or_else(|| self.nan.const_undefined());
    self.store(*d, v);
            }

            _ => unreachable!()
        }
        Ok(())
    }
}
