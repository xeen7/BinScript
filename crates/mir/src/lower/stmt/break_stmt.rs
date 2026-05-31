use diagnostics::CompileResult;
use crate::types::*;
use super::super::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_break(&mut self, label: &Option<String>) -> CompileResult<()> {
        if let Some(lbl) = label {
            if let Some(frame) = self.loop_stack.iter().rev().find(|f| f.label.as_ref() == Some(lbl)) {
                self.emit(MirInstr::Jump(frame.break_target));
            }
        } else {
            if let Some(frame) = self.loop_stack.last() {
                self.emit(MirInstr::Jump(frame.break_target));
            }
        }
        Ok(())
    }
}
