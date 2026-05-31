#!/usr/bin/env python3
import os

BASE = "crates/mir/src/lower/expr/method_call"
os.makedirs(BASE, exist_ok=True)

HEADER = """use diagnostics::CompileResult;
use hir::HirExpr;
use crate::types::*;
use super::super::LowerCtx;

"""

array_rs = HEADER + """impl<'a> LowerCtx<'a> {
    pub(super) fn lower_method_array(
        &mut self,
        obj_operand: MirOperand,
        method: &str,
        args: &[HirExpr],
    ) -> CompileResult<Option<MirOperand>> {
        let expected_args = match method {
            "push" | "join" | "concat" | "forEach" | "map" | "filter" | "find"
            | "findIndex" | "every" | "some" | "indexOf" | "includes" => 1,
            "slice" | "reduce" => 2,
            "fill" => 3,
            "pop" | "reverse" => 0,
            _ => return Ok(None),
        };

        let method_idx = self.method_indices.get(method).map(|&i| i as f64).unwrap_or(-1.0);
        let mut mir_args = vec![obj_operand];

        for i in 0..expected_args {
            if i < args.len() {
                mir_args.push(self.lower_expr(&args[i])?);
            } else {
                mir_args.push(MirOperand::ConstUndefined);
            }
        }

        mir_args.push(MirOperand::ConstNum(method_idx));

        let dest = self.fresh_reg();
        self.emit(MirInstr::CallDirect(
            dest,
            format!("__bs_call_{}", method),
            mir_args,
        ));
        Ok(Some(MirOperand::Reg(dest)))
    }
}
"""

string_rs = HEADER + """impl<'a> LowerCtx<'a> {
    pub(super) fn lower_method_string(
        &mut self,
        obj_operand: MirOperand,
        method: &str,
        args: &[HirExpr],
    ) -> CompileResult<Option<MirOperand>> {
        let expected_args = match method {
            "charAt" | "charCodeAt" | "startsWith" | "endsWith" | "split" | "repeat" => 1,
            "substring" | "replace" | "padStart" | "padEnd" => 2,
            "trim" | "toUpperCase" | "toLowerCase" => 0,
            _ => return Ok(None),
        };

        let method_idx = self.method_indices.get(method).map(|&i| i as f64).unwrap_or(-1.0);
        let mut mir_args = vec![obj_operand];

        for i in 0..expected_args {
            if i < args.len() {
                mir_args.push(self.lower_expr(&args[i])?);
            } else {
                mir_args.push(MirOperand::ConstUndefined);
            }
        }

        mir_args.push(MirOperand::ConstNum(method_idx));

        let dest = self.fresh_reg();
        self.emit(MirInstr::CallDirect(
            dest,
            format!("__bs_call_{}", method),
            mir_args,
        ));
        Ok(Some(MirOperand::Reg(dest)))
    }
}
"""

date_rs = HEADER + """impl<'a> LowerCtx<'a> {
    pub(super) fn lower_method_date(
        &mut self,
        obj_operand: MirOperand,
        method: &str,
        args: &[HirExpr],
    ) -> CompileResult<Option<MirOperand>> {
        let expected_args = match method {
            "getTime" | "getFullYear" | "getMonth" | "getDate" | "getHours" | "getMinutes" | "getSeconds" => 0,
            _ => return Ok(None),
        };

        let method_idx = self.method_indices.get(method).map(|&i| i as f64).unwrap_or(-1.0);
        let mut mir_args = vec![obj_operand];

        for i in 0..expected_args {
            if i < args.len() {
                mir_args.push(self.lower_expr(&args[i])?);
            } else {
                mir_args.push(MirOperand::ConstUndefined);
            }
        }

        mir_args.push(MirOperand::ConstNum(method_idx));

        let dest = self.fresh_reg();
        self.emit(MirInstr::CallDirect(
            dest,
            format!("__bs_call_{}", method),
            mir_args,
        ));
        Ok(Some(MirOperand::Reg(dest)))
    }
}
"""

object_rs = HEADER + """impl<'a> LowerCtx<'a> {
    pub(super) fn lower_method_object(
        &mut self,
        obj_operand: MirOperand,
        method: &str,
        args: &[HirExpr],
    ) -> CompileResult<Option<MirOperand>> {
        let expected_args = match method {
            "toString" | "valueOf" => 0,
            _ => return Ok(None),
        };

        let method_idx = self.method_indices.get(method).map(|&i| i as f64).unwrap_or(-1.0);
        let mut mir_args = vec![obj_operand];

        for i in 0..expected_args {
            if i < args.len() {
                mir_args.push(self.lower_expr(&args[i])?);
            } else {
                mir_args.push(MirOperand::ConstUndefined);
            }
        }

        mir_args.push(MirOperand::ConstNum(method_idx));

        let dest = self.fresh_reg();
        self.emit(MirInstr::CallDirect(
            dest,
            format!("__bs_call_{}", method),
            mir_args,
        ));
        Ok(Some(MirOperand::Reg(dest)))
    }
}
"""

iterator_rs = HEADER + """impl<'a> LowerCtx<'a> {
    pub(super) fn lower_method_iterator(
        &mut self,
        obj_operand: MirOperand,
        method: &str,
        args: &[HirExpr],
    ) -> CompileResult<Option<MirOperand>> {
        let expected_args = match method {
            "next" => 1,
            _ => return Ok(None),
        };

        let method_idx = self.method_indices.get(method).map(|&i| i as f64).unwrap_or(-1.0);
        let mut mir_args = vec![obj_operand];

        for i in 0..expected_args {
            if i < args.len() {
                mir_args.push(self.lower_expr(&args[i])?);
            } else {
                mir_args.push(MirOperand::ConstUndefined);
            }
        }

        mir_args.push(MirOperand::ConstNum(method_idx));

        let dest = self.fresh_reg();
        self.emit(MirInstr::CallDirect(
            dest,
            format!("__bs_call_{}", method),
            mir_args,
        ));
        Ok(Some(MirOperand::Reg(dest)))
    }
}
"""

dynamic_rs = HEADER + """impl<'a> LowerCtx<'a> {
    pub(super) fn lower_method_dynamic(
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
            let mut call_args = vec![MirOperand::Reg(fn_reg)];
            for a in args {
                call_args.push(self.lower_expr(a)?);
            }
            let dest = self.fresh_reg();
            self.emit(MirInstr::CallClosure(dest, fn_reg, call_args));
            Ok(MirOperand::Reg(dest))
        }
    }
}
"""

mod_rs = """use diagnostics::CompileResult;
use hir::HirExpr;
use crate::types::*;
use super::super::LowerCtx;

mod array;
mod string;
mod date;
mod object;
mod iterator;
mod dynamic;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_method_call(
        &mut self,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
    ) -> CompileResult<MirOperand> {
        let obj_operand = self.lower_expr(object)?;

        if let Some(res) = self.lower_method_array(obj_operand.clone(), method, args)? {
            return Ok(res);
        }
        if let Some(res) = self.lower_method_string(obj_operand.clone(), method, args)? {
            return Ok(res);
        }
        if let Some(res) = self.lower_method_date(obj_operand.clone(), method, args)? {
            return Ok(res);
        }
        if let Some(res) = self.lower_method_object(obj_operand.clone(), method, args)? {
            return Ok(res);
        }
        if let Some(res) = self.lower_method_iterator(obj_operand.clone(), method, args)? {
            return Ok(res);
        }

        self.lower_method_dynamic(obj_operand, method, args)
    }
}
"""

def write(filename, content):
    with open(f"{BASE}/{filename}", "w") as f:
        f.write(content)

write("array.rs", array_rs)
write("string.rs", string_rs)
write("date.rs", date_rs)
write("object.rs", object_rs)
write("iterator.rs", iterator_rs)
write("dynamic.rs", dynamic_rs)
write("mod.rs", mod_rs)

print("Generated method_call subdirectory files.")
