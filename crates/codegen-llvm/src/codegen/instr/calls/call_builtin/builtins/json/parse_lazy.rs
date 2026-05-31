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
    #[allow(unused_variables)]
    pub(in crate::codegen::instr) fn emit_builtin_json_parse_lazy(&mut self, d: &MirReg, args: &[MirOperand]) -> CompileResult<()> {
let json_parse = self.module.get_function("__bs_json_parse_lazy").unwrap();
    if let MirOperand::ConstStr(s) = &args[0] {
        let global_str = self.builder.build_global_string_ptr(s, "json_str").unwrap();
        let ptr_val = global_str.as_pointer_value();
        let len_val = self.i32_ty.const_int(s.len() as u64, false);
        let rv = self.builder.build_call(json_parse, &[ptr_val.into(), len_val.into()], "json_parse_lazy").unwrap();
        let v = rv.try_as_basic_value().basic().unwrap().into_int_value();
        self.store(*d, v);
    } else {
        return Err(CompileError::Codegen { message: "JsonParseLazy arg must be ConstStr".into() });
    }
        Ok(())
    }
}
