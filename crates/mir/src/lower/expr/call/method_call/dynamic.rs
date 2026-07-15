use diagnostics::CompileResult;
use hir::HirExpr;
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_method_dynamic(
        &mut self,
        obj_operand: MirOperand,
        method: &str,
        args: &[HirExpr],
    ) -> CompileResult<MirOperand> {
        let obj_reg = match obj_operand {
            MirOperand::Reg(r) => r,
            _ => {
                let r = self.fresh_reg();
                self.emit(MirInstr::Move(r, obj_operand));
                r
            }
        };

        let has_spread = args.iter().any(|a| matches!(a, HirExpr::Spread(_)));
        if has_spread {
            let args_array = self.fresh_reg();
            self.emit(MirInstr::CallDirect(args_array, "__bs_array_new".to_string(), vec![]));

            // Prepend the receiver obj
            let unused = self.fresh_reg();
            self.emit(MirInstr::CallDirect(
                unused,
                "__bs_array_push".to_string(),
                vec![MirOperand::Reg(args_array), MirOperand::Reg(obj_reg)],
            ));

            for a in args {
                if let HirExpr::Spread(inner) = a {
                    let op = self.lower_expr(inner)?;
                    let unused = self.fresh_reg();
                    self.emit(MirInstr::CallDirect(
                        unused,
                        "__bs_array_push_spread".to_string(),
                        vec![MirOperand::Reg(args_array), op],
                    ));
                } else {
                    let op = self.lower_expr(a)?;
                    let unused = self.fresh_reg();
                    self.emit(MirInstr::CallDirect(
                        unused,
                        "__bs_array_push".to_string(),
                        vec![MirOperand::Reg(args_array), op],
                    ));
                }
            }

            let dest = self.fresh_reg();
            if let Some(&method_idx) = self.method_indices.get(method) {
                self.emit(MirInstr::CallDirect(
                    dest,
                    "__bs_vcall_apply".to_string(),
                    vec![
                        MirOperand::Reg(obj_reg),
                        MirOperand::ConstNum(method_idx as f64),
                        MirOperand::Reg(args_array),
                    ],
                ));
            } else {
                let fn_reg = self.fresh_reg();
                self.emit(MirInstr::LoadProp(fn_reg, obj_reg, method.to_string()));
                self.emit(MirInstr::CallDirect(
                    dest,
                    "__bs_call_apply".to_string(),
                    vec![
                        MirOperand::Reg(fn_reg),
                        MirOperand::ConstUndefined,
                        MirOperand::Reg(args_array),
                    ],
                ));
            }
            return Ok(MirOperand::Reg(dest));
        }

        if let Some(&method_idx) = self.method_indices.get(method) {
            let mut mir_args = vec![MirOperand::Reg(obj_reg)];
            for a in args {
                mir_args.push(self.lower_expr(a)?);
            }

            let dest = self.fresh_reg();
            self.emit(MirInstr::CallVTable(dest, obj_reg, method_idx, mir_args));
            Ok(MirOperand::Reg(dest))
        } else {
            // Fallback: load method as a closure from dynamic property and call it
            let fn_reg = self.fresh_reg();
            self.emit(MirInstr::LoadProp(fn_reg, obj_reg, method.to_string()));
            let mut call_args = vec![MirOperand::Reg(fn_reg), MirOperand::Reg(obj_reg)];
            for a in args {
                call_args.push(self.lower_expr(a)?);
            }
            let dest = self.fresh_reg();
            self.emit(MirInstr::CallClosure(dest, fn_reg, call_args));
            Ok(MirOperand::Reg(dest))
        }
    }
}
