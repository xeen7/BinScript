#![allow(unused_imports)]
#![allow(unused_unsafe)]
use inkwell::values::BasicMetadataValueEnum;
use inkwell::IntPredicate;
use inkwell::FloatPredicate;
use mir::types::*;
use mir::BuiltinFn;
use diagnostics::{CompileError, CompileResult};

use crate::codegen::LlvmCodegen;

mod array;
mod console;
mod generator;
mod json;
mod promise;
mod timer;

impl<'ctx> LlvmCodegen<'ctx> {
    pub(in crate::codegen::instr) fn emit_instr_call_builtin(&mut self, instr: &MirInstr) -> CompileResult<()> {
        let MirInstr::CallBuiltin(d, builtin, args) = instr else { unreachable!() };
        match builtin {
            BuiltinFn::ConsoleLog => self.emit_builtin_console_log(d, args)?,
            BuiltinFn::GeneratorNext => self.emit_builtin_generator_next(d, args)?,
            BuiltinFn::GeneratorIsDone => self.emit_builtin_generator_is_done(d, args)?,
            BuiltinFn::PromiseAll2 => self.emit_builtin_promise_all_2(d, args)?,
            BuiltinFn::PromiseRace2 => self.emit_builtin_promise_race_2(d, args)?,
            BuiltinFn::JsonParseLazy => self.emit_builtin_json_parse_lazy(d, args)?,
            BuiltinFn::Sleep => self.emit_builtin_sleep(d, args)?,
            BuiltinFn::ArrayNew => self.emit_builtin_array_new(d, args)?,
            BuiltinFn::ArrayFrom => self.emit_builtin_array_from(d, args)?,
            BuiltinFn::ArrayPush => self.emit_builtin_array_push(d, args)?,
            BuiltinFn::ArrayPop => self.emit_builtin_array_pop(d, args)?,
            BuiltinFn::ArrayGet => self.emit_builtin_array_get(d, args)?,
            BuiltinFn::ArraySet => self.emit_builtin_array_set(d, args)?,
            BuiltinFn::ArrayLength => self.emit_builtin_array_length(d, args)?,
            BuiltinFn::ArraySlice => self.emit_builtin_array_slice(d, args)?,
            BuiltinFn::ArrayIndexOf => self.emit_builtin_array_index_of(d, args)?,
            BuiltinFn::ArrayIncludes => self.emit_builtin_array_includes(d, args)?,
            BuiltinFn::ArrayJoin => self.emit_builtin_array_join(d, args)?,
            BuiltinFn::ArrayReverse => self.emit_builtin_array_reverse(d, args)?,
            BuiltinFn::ArrayConcat => self.emit_builtin_array_concat(d, args)?,
            BuiltinFn::ArrayFill => self.emit_builtin_array_fill(d, args)?,
            BuiltinFn::ArrayIsArray => self.emit_builtin_array_is_array(d, args)?,
            BuiltinFn::ArrayForEach => self.emit_builtin_array_for_each(d, args)?,
            BuiltinFn::ArrayMap => self.emit_builtin_array_map(d, args)?,
            BuiltinFn::ArrayFilter => self.emit_builtin_array_filter(d, args)?,
            BuiltinFn::ArrayFind => self.emit_builtin_array_find(d, args)?,
            BuiltinFn::ArrayFindIndex => self.emit_builtin_array_find_index(d, args)?,
            BuiltinFn::ArrayEvery => self.emit_builtin_array_every(d, args)?,
            BuiltinFn::ArraySome => self.emit_builtin_array_some(d, args)?,
            BuiltinFn::ArrayReduce => self.emit_builtin_array_reduce(d, args)?,
        }
        Ok(())
    }
}
