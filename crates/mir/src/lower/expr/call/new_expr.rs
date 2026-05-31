use diagnostics::CompileResult;
use hir::HirExpr;
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_new(
        &mut self,
        class_name: &str,
        args: &[HirExpr],
    ) -> CompileResult<MirOperand> {
        let mut mir_args = Vec::new();
        for a in args {
            mir_args.push(self.lower_expr(a)?);
        }
        let obj_reg = self.fresh_reg();
        self.reg_shapes.insert(obj_reg, class_name.to_string());

        self.emit(MirInstr::Alloc(obj_reg, class_name.to_string()));

        let mut ctor_args = vec![MirOperand::Reg(obj_reg)];
        ctor_args.extend(mir_args);

        let unused = self.fresh_reg();
        let ctor_name = format!("__bs_class_{}_constructor", class_name);
        self.emit(MirInstr::CallDirect(unused, ctor_name, ctor_args));

        Ok(MirOperand::Reg(obj_reg))
    }
}
