use mir::types::MirInstr;
use diagnostics::CompileResult;

use crate::codegen::LlvmCodegen;

impl<'ctx> LlvmCodegen<'ctx> {
    pub(in crate::codegen::instr) fn emit_instr_borrow(&mut self, instr: &MirInstr) -> CompileResult<()> {
        if let MirInstr::Borrow(dest, src) | MirInstr::BorrowMut(dest, src) = instr {
            // A borrow is just copying the pointer value without affecting ownership.
            // Since our LLVM registers just hold scalar values, a Move is sufficient.
            let val = self.val(&mir::types::MirOperand::Reg(*src))?;
            self.store(*dest, val);
        }
        Ok(())
    }
}
