use diagnostics::CompileResult;
use crate::types::*;
use super::super::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_continue(&mut self, label: &Option<String>) -> CompileResult<()> {
        if let Some(lbl) = label {
            if let Some(frame) = self.loop_stack.iter().rev().find(|f| f.label.as_ref() == Some(lbl)) {
                if let Some(continue_target) = frame.continue_target {
                    self.emit(MirInstr::Jump(continue_target));
                }
            }
        } else {
            if let Some(continue_target) = self.loop_stack.iter().rev().filter_map(|f| f.continue_target).next() {
                self.emit(MirInstr::Jump(continue_target));
            }
        }
        Ok(())
    }
}
