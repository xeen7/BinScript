use diagnostics::CompileResult;
use hir::HirExpr;
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_array_lit(&mut self, elems: &[HirExpr]) -> CompileResult<MirOperand> {
        let dest = self.fresh_reg();
        self.emit(MirInstr::CallDirect(dest, "__bs_array_new".to_string(), vec![]));
        for elem in elems {
            if let HirExpr::Spread(inner) = elem {
                let operand = self.lower_expr(inner)?;
                let unused = self.fresh_reg();
                self.emit(MirInstr::CallDirect(
                    unused,
                    "__bs_array_push_spread".to_string(),
                    vec![MirOperand::Reg(dest), operand],
                ));
            } else {
                let operand = self.lower_expr(elem)?;
                let unused = self.fresh_reg();
                self.emit(MirInstr::CallDirect(
                    unused,
                    "__bs_array_push".to_string(),
                    vec![MirOperand::Reg(dest), operand],
                ));
            }
        }
        Ok(MirOperand::Reg(dest))
    }
}
