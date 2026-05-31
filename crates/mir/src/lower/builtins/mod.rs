use diagnostics::CompileResult;
use crate::types::*;
use super::LowerCtx;

mod console;
mod promise;
mod number;
mod object;
mod string;
mod date;
mod math;
mod json;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_builtin_member_call(
        &mut self,
        object: &str,
        method: &str,
        mir_args: Vec<MirOperand>,
        dest: MirReg,
    ) -> CompileResult<bool> {
        match object {
            "console" => self.lower_builtin_console(method, mir_args, dest),
            "Promise"  => self.lower_builtin_promise(method, mir_args, dest),
            "Number"   => self.lower_builtin_number(method, mir_args, dest),
            "Object"   => self.lower_builtin_object(method, mir_args, dest),
            "String"   => self.lower_builtin_string(method, mir_args, dest),
            "Date"     => self.lower_builtin_date(method, mir_args, dest),
            "Math"     => self.lower_builtin_math(method, mir_args, dest),
            "JSON"     => self.lower_builtin_json(method, mir_args, dest),
            _          => Ok(false),
        }
    }
}
pub mod builtin_fn;
pub use builtin_fn::BuiltinFn;
