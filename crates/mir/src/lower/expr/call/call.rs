use diagnostics::CompileResult;
use hir::HirExpr;
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_call(
        &mut self,
        callee: &HirExpr,
        args: &[HirExpr],
    ) -> CompileResult<MirOperand> {
        let mir_args: Vec<MirOperand> = args
            .iter()
            .map(|a| self.lower_expr(a))
            .collect::<CompileResult<_>>()?;
        let dest = self.fresh_reg();
        match callee {
            HirExpr::GlobalRef(name) => {
                if let Some(captures) = self.global_fn_captures.get(name) {
                    if !captures.is_empty() {
                        if let Some(&bid) = self.global_fn_bindings.get(name) {
                            if let Some(&callee_reg) = self.bindings.get(&bid) {
                                let mut call_args = vec![MirOperand::Reg(callee_reg)];
                                call_args.extend(mir_args);
                                self.emit(MirInstr::CallClosure(dest, callee_reg, call_args));
                                return Ok(MirOperand::Reg(dest));
                            }
                        }
                    }
                }
                if name == "__bs_mock_push" {
                    if let Some(MirOperand::Reg(r)) = mir_args.get(0) {
                        self.emit(MirInstr::ScopeGuardPush {
                            scope_id: 1,
                            reg: *r,
                            release_fn: "__bs_mock_release".to_string(),
                        });
                        return Ok(MirOperand::Reg(dest));
                    }
                }
                if name == "__bs_mock_cancel" {
                    if let Some(MirOperand::Reg(r)) = mir_args.get(0) {
                        self.emit(MirInstr::ScopeGuardCancel {
                            scope_id: 1,
                            reg: *r,
                        });
                        return Ok(MirOperand::Reg(dest));
                    }
                }
                if name == "__bs_mock_flush" {
                    self.emit(MirInstr::ScopeGuardFlushTo {
                        current_scope: 1,
                        target_scope: 0,
                    });
                    return Ok(MirOperand::Reg(dest));
                }

                let fn_name = if name == "parseInt" {
                    if mir_args.len() == 1 {
                        "__bs_parseInt_1".to_string()
                    } else {
                        "__bs_parseInt_2".to_string()
                    }
                } else {
                    self.func_names
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| {
                            if name.starts_with("__bs_") {
                                name.clone()
                            } else {
                                format!("__bs_{}", name)
                            }
                        })
                };
                self.emit(MirInstr::CallDirect(dest, fn_name, mir_args));
            }
            _ => {
                let callee_op = self.lower_expr(callee)?;
                let callee_reg = match callee_op {
                    MirOperand::Reg(r) => r,
                    other => {
                        let r = self.fresh_reg();
                        self.emit(MirInstr::Move(r, other));
                        r
                    }
                };
                let mut call_args = vec![MirOperand::Reg(callee_reg)];
                call_args.extend(mir_args);
                self.emit(MirInstr::CallClosure(dest, callee_reg, call_args));
            }
        }
        Ok(MirOperand::Reg(dest))
    }
}
