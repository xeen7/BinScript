use diagnostics::CompileResult;
use hir::HirExpr;
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_await(&mut self, inner: &HirExpr) -> CompileResult<MirOperand> {
        let v = self.lower_expr(inner)?;
        if self.is_async_generator {
            return Ok(v);
        }

        let yield_idx = self.num_yield_points;
        self.num_yield_points += 1;

        let mut saves = Vec::new();
        for r in 0..self.next_reg {
            saves.push(r);
        }
        self.yield_saves.push(saves);

        self.emit(MirInstr::Suspend(yield_idx, v));
        let dest = self.fresh_reg();
        self.emit(MirInstr::Resume(dest, yield_idx));
        Ok(MirOperand::Reg(dest))
    }
}
