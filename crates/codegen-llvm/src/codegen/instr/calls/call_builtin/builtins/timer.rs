use inkwell::values::BasicMetadataValueEnum;
use mir::types::{MirOperand, MirReg};
use diagnostics::CompileResult;

use crate::codegen::LlvmCodegen;

impl<'ctx> LlvmCodegen<'ctx> {
    #[allow(unused_variables)]
    pub(in crate::codegen::instr) fn emit_builtin_sleep(
        &mut self,
        dest: &MirReg,
        args: &[MirOperand],
    ) -> CompileResult<()> {
        let ms_val = self.val(&args[0])?;
        
        let timeout_fn = self.funcs["__bs_set_timeout"];
        
        let promise_val = self.builder.build_call(
            timeout_fn,
            &[ms_val.into()],
            "call_set_timeout",
        ).unwrap().try_as_basic_value().basic().unwrap().into_int_value();
        
        self.store(*dest, promise_val);
        
        Ok(())
    }
}
