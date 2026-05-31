#!/usr/bin/env python3
"""
Script to split crates/mir/src/lower/{expr,stmt,builtins}.rs into granular sub-directories.
"""
import os

BASE = "crates/mir/src/lower"

HEADER_EXPR = """use diagnostics::{CompileError, CompileResult};
use hir::{HirExpr, Literal, UnaryOp as HUnaryOp, BinOp as HBinOp};
use crate::builtins::BuiltinFn;
use crate::types::*;
use super::super::LowerCtx;
"""

HEADER_STMT = """use diagnostics::{CompileError, CompileResult};
use hir::{HirStmt, HirExpr};
use crate::types::*;
use super::super::{LowerCtx, LoopStackFrame};
use crate::builtins::BuiltinFn;
"""

HEADER_BUILTIN = """use diagnostics::{CompileError, CompileResult};
use crate::builtins::BuiltinFn;
use crate::types::*;
use super::super::LowerCtx;
"""

# ──────────────────────────────────────────────────────────────────────────────
# Expression files
# ──────────────────────────────────────────────────────────────────────────────

expr_files = {}

expr_files["lit.rs"] = HEADER_EXPR + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_lit(&mut self, lit: &Literal) -> CompileResult<MirOperand> {
        Ok(match lit {
            Literal::Number(n) => MirOperand::ConstNum(*n),
            Literal::String(s) => MirOperand::ConstStr(s.clone()),
            Literal::Bool(b) => MirOperand::ConstBool(*b),
            Literal::Null => MirOperand::ConstNull,
            Literal::Undefined => MirOperand::ConstUndefined,
        })
    }
}
"""

expr_files["var.rs"] = HEADER_EXPR + """
use hir::BindingId;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_var(&mut self, bid: &BindingId) -> CompileResult<MirOperand> {
        if let Some(&reg) = self.bindings.get(bid) {
            if self.capture_cells.contains(bid) {
                let dest = self.fresh_reg();
                self.emit(MirInstr::LoadField(dest, reg, 0));
                Ok(MirOperand::Reg(dest))
            } else {
                Ok(MirOperand::Reg(reg))
            }
        } else {
            Err(CompileError::Lowering {
                message: format!("Unresolved binding {}", bid),
            })
        }
    }
}
"""

expr_files["bin_op.rs"] = HEADER_EXPR + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_bin_op(
        &mut self,
        op: &HBinOp,
        left: &HirExpr,
        right: &HirExpr,
    ) -> CompileResult<MirOperand> {
        if matches!(op, HBinOp::And | HBinOp::Or | HBinOp::NullishCoalescing) {
            let l = self.lower_expr(left)?;
            let dest = self.fresh_reg();
            let eval_r_bb = self.fresh_block();
            let merge_bb = self.fresh_block();

            self.emit(MirInstr::Move(dest, l.clone()));
            if let HBinOp::And = op {
                // AND: if left is true, evaluate right; else keep left (falsy)
                self.emit(MirInstr::Branch(l, eval_r_bb, merge_bb));
            } else if let HBinOp::Or = op {
                // OR: if left is true, keep left (truthy); else evaluate right
                self.emit(MirInstr::Branch(l, merge_bb, eval_r_bb));
            } else {
                // Nullish Coalescing (??): if left is null/undefined, evaluate right; else keep left
                let cond_reg = self.fresh_reg();
                self.emit(MirInstr::CallDirect(cond_reg, "__bs_is_nullish".to_string(), vec![l]));
                self.emit(MirInstr::Branch(MirOperand::Reg(cond_reg), eval_r_bb, merge_bb));
            }

            self.switch_to(eval_r_bb);
            let r = self.lower_expr(right)?;
            self.emit(MirInstr::Move(dest, r));
            self.emit(MirInstr::Jump(merge_bb));

            self.switch_to(merge_bb);
            return Ok(MirOperand::Reg(dest));
        }

        let l = self.lower_expr(left)?;
        let r = self.lower_expr(right)?;
        let dest = self.fresh_reg();
        let instr = match op {
            HBinOp::Add => MirInstr::Add(dest, l, r),
            HBinOp::Sub => MirInstr::Sub(dest, l, r),
            HBinOp::Mul => MirInstr::Mul(dest, l, r),
            HBinOp::Div => MirInstr::Div(dest, l, r),
            HBinOp::Mod => MirInstr::Mod(dest, l, r),
            HBinOp::Exp => MirInstr::Exp(dest, l, r),
            HBinOp::Eq => MirInstr::Eq(dest, l, r),
            HBinOp::Ne => MirInstr::Ne(dest, l, r),
            HBinOp::StrictEq => MirInstr::StrictEq(dest, l, r),
            HBinOp::StrictNe => MirInstr::StrictNe(dest, l, r),
            HBinOp::Lt => MirInstr::Lt(dest, l, r),
            HBinOp::Le => MirInstr::Le(dest, l, r),
            HBinOp::Gt => MirInstr::Gt(dest, l, r),
            HBinOp::Ge => MirInstr::Ge(dest, l, r),
            HBinOp::In => MirInstr::In(dest, l, r),
            HBinOp::BitAnd => MirInstr::BitAnd(dest, l, r),
            HBinOp::BitOr => MirInstr::BitOr(dest, l, r),
            HBinOp::BitXor => MirInstr::BitXor(dest, l, r),
            HBinOp::Shl => MirInstr::Shl(dest, l, r),
            HBinOp::Shr => MirInstr::Shr(dest, l, r),
            HBinOp::UShr => MirInstr::UShr(dest, l, r),
            HBinOp::And | HBinOp::Or | HBinOp::NullishCoalescing => unreachable!(),
        };
        self.emit(instr);
        Ok(MirOperand::Reg(dest))
    }
}
"""

expr_files["unary_op.rs"] = HEADER_EXPR + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_unary_op(
        &mut self,
        op: &HUnaryOp,
        arg: &HirExpr,
    ) -> CompileResult<MirOperand> {
        let v = self.lower_expr(arg)?;
        let dest = self.fresh_reg();
        match op {
            HUnaryOp::Plus => self.emit(MirInstr::Plus(dest, v)),
            HUnaryOp::Neg => self.emit(MirInstr::Neg(dest, v)),
            HUnaryOp::Not => self.emit(MirInstr::Not(dest, v)),
            HUnaryOp::Typeof => self.emit(MirInstr::CallDirect(dest, "__bs_typeof".to_string(), vec![v])),
            HUnaryOp::BitNot => self.emit(MirInstr::BitNot(dest, v)),
            HUnaryOp::Void => self.emit(MirInstr::Move(dest, MirOperand::ConstUndefined)),
        }
        Ok(MirOperand::Reg(dest))
    }
}
"""

expr_files["call.rs"] = HEADER_EXPR + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_call(
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
"""

expr_files["member_call.rs"] = HEADER_EXPR + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_member_call(
        &mut self,
        object: &str,
        method: &str,
        args: &[HirExpr],
    ) -> CompileResult<MirOperand> {
        let mir_args: Vec<MirOperand> = args
            .iter()
            .map(|a| self.lower_expr(a))
            .collect::<CompileResult<_>>()?;
        let dest = self.fresh_reg();
        if self.lower_builtin_member_call(object, method, mir_args, dest)? {
            return Ok(MirOperand::Reg(dest));
        }
        Err(CompileError::Lowering {
            message: format!("{}.{}() not supported", object, method),
        })
    }
}
"""

expr_files["assign.rs"] = HEADER_EXPR + """
use hir::BindingId;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_assign(
        &mut self,
        target: &BindingId,
        value: &HirExpr,
    ) -> CompileResult<MirOperand> {
        let val = self.lower_expr(value)?;
        // Look up the binding; if it's unknown (e.g. a temp from object literal lowering),
        // auto-allocate a fresh register for it so it can be used later.
        let reg = if let Some(&existing) = self.bindings.get(target) {
            existing
        } else {
            let fresh = self.fresh_reg();
            self.bindings.insert(*target, fresh);
            fresh
        };
        if self.capture_cells.contains(target) {
            self.emit(MirInstr::StoreField(reg, 0, val.clone()));
        } else {
            if let MirOperand::Reg(src_reg) = &val {
                if let Some(shape) = self.reg_shapes.get(&src_reg).cloned() {
                    self.reg_shapes.insert(reg, shape);
                }
            }
            self.emit(MirInstr::Move(reg, val.clone()));
        }
        Ok(val)
    }
}
"""

expr_files["ternary.rs"] = HEADER_EXPR + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_ternary(
        &mut self,
        cond: &HirExpr,
        then_expr: &HirExpr,
        else_expr: &HirExpr,
    ) -> CompileResult<MirOperand> {
        let cv = self.lower_expr(cond)?;
        let dest = self.fresh_reg();
        let then_bb = self.fresh_block();
        let else_bb = self.fresh_block();
        let merge_bb = self.fresh_block();

        self.emit(MirInstr::Branch(cv, then_bb, else_bb));

        self.switch_to(then_bb);
        let tv = self.lower_expr(then_expr)?;
        self.emit(MirInstr::Move(dest, tv));
        self.emit(MirInstr::Jump(merge_bb));

        self.switch_to(else_bb);
        let ev = self.lower_expr(else_expr)?;
        self.emit(MirInstr::Move(dest, ev));
        self.emit(MirInstr::Jump(merge_bb));

        self.switch_to(merge_bb);
        Ok(MirOperand::Reg(dest))
    }
}
"""

expr_files["delete_prop.rs"] = HEADER_EXPR + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_delete_prop(
        &mut self,
        object: &HirExpr,
        property: &HirExpr,
    ) -> CompileResult<MirOperand> {
        let obj = self.lower_expr(object)?;
        let prop = self.lower_expr(property)?;
        let dest = self.fresh_reg();
        self.emit(MirInstr::DeleteProp(dest, obj, prop));
        Ok(MirOperand::Reg(dest))
    }
}
"""

expr_files["yield_expr.rs"] = HEADER_EXPR + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_yield(
        &mut self,
        arg: &Option<Box<HirExpr>>,
        delegate: bool,
    ) -> CompileResult<MirOperand> {
        if delegate {
            let iter_val = match arg {
                Some(e) => self.lower_expr(e)?,
                None => MirOperand::ConstUndefined,
            };
            let iter_reg = self.fresh_reg();
            self.emit(MirInstr::Move(iter_reg, iter_val));

            let sent_reg = self.fresh_reg();
            self.emit(MirInstr::Move(sent_reg, MirOperand::ConstUndefined));

            let result_reg = self.fresh_reg();

            let cond_bb = self.fresh_block();
            let body_bb = self.fresh_block();
            let exit_bb = self.fresh_block();

            self.emit(MirInstr::Jump(cond_bb));
            self.switch_to(cond_bb);

            let next_val_reg = self.fresh_reg();
            self.emit(MirInstr::CallBuiltin(
                next_val_reg,
                BuiltinFn::GeneratorNext,
                vec![MirOperand::Reg(iter_reg), MirOperand::Reg(sent_reg)],
            ));

            let is_done_reg = self.fresh_reg();
            self.emit(MirInstr::CallBuiltin(
                is_done_reg,
                BuiltinFn::GeneratorIsDone,
                vec![MirOperand::Reg(iter_reg)],
            ));
            self.emit(MirInstr::Branch(MirOperand::Reg(is_done_reg), exit_bb, body_bb));

            // Body
            self.switch_to(body_bb);

            let yield_idx = self.num_yield_points;
            self.num_yield_points += 1;
            let mut saves = Vec::new();
            for r in 0..self.next_reg {
                saves.push(r);
            }
            self.yield_saves.push(saves);

            self.emit(MirInstr::Suspend(yield_idx, MirOperand::Reg(next_val_reg)));
            self.emit(MirInstr::Resume(sent_reg, yield_idx));
            self.emit(MirInstr::Jump(cond_bb));

            // Exit
            self.switch_to(exit_bb);
            self.emit(MirInstr::Move(result_reg, MirOperand::Reg(next_val_reg)));

            Ok(MirOperand::Reg(result_reg))
        } else {
            let v = match arg {
                Some(e) => self.lower_expr(e)?,
                None => MirOperand::ConstUndefined,
            };
            let yield_idx = self.num_yield_points;
            self.num_yield_points += 1;

            // Save all registers assigned up to this point
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
}
"""

expr_files["await_expr.rs"] = HEADER_EXPR + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_await(&mut self, inner: &HirExpr) -> CompileResult<MirOperand> {
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
"""

expr_files["seq.rs"] = HEADER_EXPR + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_seq(&mut self, exprs: &[HirExpr]) -> CompileResult<MirOperand> {
        let mut last = MirOperand::ConstUndefined;
        for e in exprs {
            last = self.lower_expr(e)?;
        }
        Ok(last)
    }
}
"""

expr_files["global_ref.rs"] = HEADER_EXPR + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_global_ref(&mut self, name: &str) -> CompileResult<MirOperand> {
        if name.starts_with("__bs_class_val_") {
            let dest = self.fresh_reg();
            let class_name = name["__bs_class_val_".len()..].to_string();
            self.emit(MirInstr::LoadGlobal(dest, name.to_string()));
            // Track as class constructor for static getter/setter resolution
            if self.classes.contains_key(&class_name) {
                self.class_constructors.insert(dest, class_name);
            }
            return Ok(MirOperand::Reg(dest));
        }
        match name {
            "NaN" => Ok(MirOperand::ConstNum(std::f64::NAN)),
            "Infinity" => Ok(MirOperand::ConstNum(std::f64::INFINITY)),
            "undefined" => Ok(MirOperand::ConstUndefined),
            "globalThis" => {
                let dest = self.fresh_reg();
                self.emit(MirInstr::CallDirect(dest, "__bs_get_globalThis".to_string(), vec![]));
                Ok(MirOperand::Reg(dest))
            }
            "Symbol" => {
                let dest = self.fresh_reg();
                self.emit(MirInstr::CallDirect(dest, "__bs_get_Symbol_global".to_string(), vec![]));
                Ok(MirOperand::Reg(dest))
            }
            _ => Ok(MirOperand::ConstUndefined),
        }
    }
}
"""

expr_files["json_tape.rs"] = HEADER_EXPR + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_json_tape(&mut self, bytes: &[u8]) -> CompileResult<MirOperand> {
        let s = String::from_utf8_lossy(bytes).into_owned();
        let dest = self.fresh_reg();
        // Emitting it as a builtin/intrinsic call. JsonParseLazy is handled in codegen.
        self.emit(MirInstr::CallBuiltin(dest, BuiltinFn::JsonParseLazy, vec![MirOperand::ConstStr(s)]));
        Ok(MirOperand::Reg(dest))
    }
}
"""

expr_files["new_expr.rs"] = HEADER_EXPR + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_new(
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
"""

expr_files["member_get.rs"] = HEADER_EXPR + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_member_get(
        &mut self,
        object: &HirExpr,
        property: &str,
    ) -> CompileResult<MirOperand> {
        let obj_operand = self.lower_expr(object)?;
        let obj_reg = match obj_operand {
            MirOperand::Reg(r) => r,
            _ => {
                let r = self.fresh_reg();
                self.emit(MirInstr::Move(r, obj_operand));
                r
            }
        };

        let dest = self.fresh_reg();
        if let Some(shape) = self.reg_shapes.get(&obj_reg) {
            if self.has_getter(shape, property) {
                let getter_name = format!("__get_{}", property);
                if let Some(&method_idx) = self.method_indices.get(&getter_name) {
                    let mir_args = vec![MirOperand::Reg(obj_reg)];
                    self.emit(MirInstr::CallVTable(dest, obj_reg, method_idx, mir_args));
                    return Ok(MirOperand::Reg(dest));
                }
            }
            if let Some(index) = self.get_field_index(shape, property) {
                self.emit(MirInstr::LoadField(dest, obj_reg, index));
                return Ok(MirOperand::Reg(dest));
            }
        }
        // Check for static getter on class constructor
        if let Some(ctor_class) = self.class_constructors.get(&obj_reg).cloned() {
            if self.has_static_getter(&ctor_class, property) {
                let getter_prop = format!("__get_{}", property);
                let closure_reg = self.fresh_reg();
                self.emit(MirInstr::LoadProp(closure_reg, obj_reg, getter_prop));
                self.emit(MirInstr::CallClosure(dest, closure_reg, vec![MirOperand::Reg(closure_reg)]));
                return Ok(MirOperand::Reg(dest));
            }
        }

        // Fallback to dynamic property get (for JsonTape and untyped objects)
        self.emit(MirInstr::LoadProp(dest, obj_reg, property.to_string()));
        Ok(MirOperand::Reg(dest))
    }
}
"""

expr_files["member_set.rs"] = HEADER_EXPR + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_member_set(
        &mut self,
        object: &HirExpr,
        property: &str,
        value: &HirExpr,
    ) -> CompileResult<MirOperand> {
        let obj_operand = self.lower_expr(object)?;
        let obj_reg = match obj_operand {
            MirOperand::Reg(r) => r,
            _ => {
                let r = self.fresh_reg();
                self.emit(MirInstr::Move(r, obj_operand));
                r
            }
        };

        let val = self.lower_expr(value)?;

        if let Some(shape) = self.reg_shapes.get(&obj_reg) {
            if self.has_setter(shape, property) {
                let setter_name = format!("__set_{}", property);
                if let Some(&method_idx) = self.method_indices.get(&setter_name) {
                    let dest = self.fresh_reg();
                    let mir_args = vec![MirOperand::Reg(obj_reg), val.clone()];
                    self.emit(MirInstr::CallVTable(dest, obj_reg, method_idx, mir_args));
                    return Ok(val);
                }
            }
            if let Some(index) = self.get_field_index(shape, property) {
                self.emit(MirInstr::StoreField(obj_reg, index, val.clone()));
                return Ok(val);
            }
        }

        // Check for static setter on class constructor
        if let Some(ctor_class) = self.class_constructors.get(&obj_reg).cloned() {
            if self.has_static_setter(&ctor_class, property) {
                let setter_prop = format!("__set_{}", property);
                let closure_reg = self.fresh_reg();
                self.emit(MirInstr::LoadProp(closure_reg, obj_reg, setter_prop));
                let dest = self.fresh_reg();
                self.emit(MirInstr::CallClosure(dest, closure_reg, vec![MirOperand::Reg(closure_reg), val.clone()]));
                return Ok(val);
            }
        }

        // Fallback to dynamic property set
        self.emit(MirInstr::StoreProp(obj_reg, property.to_string(), val.clone()));
        Ok(val)
    }
}
"""

expr_files["method_call.rs"] = HEADER_EXPR + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_method_call(
        &mut self,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
    ) -> CompileResult<MirOperand> {
        let obj_operand = self.lower_expr(object)?;

        let is_builtin = matches!(
            method,
            "push" | "pop" | "slice" | "indexOf" | "includes" | "join" | "reverse" |
            "concat" | "fill" | "forEach" | "map" | "filter" | "find" | "findIndex" |
            "every" | "some" | "reduce" | "charAt" | "charCodeAt" | "startsWith" |
            "endsWith" | "substring" | "split" | "trim" | "toUpperCase" | "toLowerCase" |
            "replace" | "repeat" | "padStart" | "padEnd" | "getTime" | "getFullYear" |
            "getMonth" | "getDate" | "getHours" | "getMinutes" | "getSeconds" | "toString" | "valueOf" | "next"
        );

        if is_builtin {
            let expected_args = match method {
                "push" | "includes" | "join" | "concat" | "forEach" | "map" | "filter" | "find"
                | "findIndex" | "every" | "some" | "charAt" | "charCodeAt" | "startsWith"
                | "endsWith" | "split" | "repeat" | "indexOf" | "next" => 1,
                "slice" | "reduce" | "substring" | "replace" | "padStart" | "padEnd" => 2,
                "fill" => 3,
                _ => 0, // pop, reverse, trim, toUpperCase, toLowerCase, and all Date getters, toString, valueOf
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
            Ok(MirOperand::Reg(dest))
        } else {
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
}
"""

expr_files["instance_of.rs"] = HEADER_EXPR + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_instance_of(
        &mut self,
        expr: &HirExpr,
        class_name: &str,
    ) -> CompileResult<MirOperand> {
        let ev = self.lower_expr(expr)?;
        let shape_id = *self.class_shapes.get(class_name).ok_or_else(|| {
            CompileError::Lowering {
                message: format!("Class '{}' in instanceof check is not defined", class_name),
            }
        })?;
        let dest = self.fresh_reg();
        self.emit(MirInstr::CallDirect(
            dest,
            "__bs_instanceof".to_string(),
            vec![ev, MirOperand::ConstNum(shape_id as f64)],
        ));
        Ok(MirOperand::Reg(dest))
    }
}
"""

expr_files["closure.rs"] = HEADER_EXPR + """
use hir::BindingId;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_closure(
        &mut self,
        func_id: &usize,
        captures: &[BindingId],
    ) -> CompileResult<MirOperand> {
        let mut mir_caps = Vec::new();
        for &bid in captures {
            if let Some(&reg) = self.bindings.get(&bid) {
                mir_caps.push(MirOperand::Reg(reg));
            } else {
                return Err(CompileError::Lowering {
                    message: format!("Captured binding {} not resolved in bindings", bid),
                });
            }
        }
        let dest = self.fresh_reg();
        self.emit(MirInstr::AllocClosure(dest, *func_id, mir_caps));
        Ok(MirOperand::Reg(dest))
    }
}
"""

expr_files["array_lit.rs"] = HEADER_EXPR + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_array_lit(&mut self, elems: &[HirExpr]) -> CompileResult<MirOperand> {
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
"""

expr_files["index_get.rs"] = HEADER_EXPR + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_index_get(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
    ) -> CompileResult<MirOperand> {
        let obj_operand = self.lower_expr(object)?;
        let idx_operand = self.lower_expr(index)?;
        let dest = self.fresh_reg();
        self.emit(MirInstr::CallDirect(
            dest,
            "__bs_index_get".to_string(),
            vec![obj_operand, idx_operand],
        ));
        Ok(MirOperand::Reg(dest))
    }
}
"""

expr_files["index_set.rs"] = HEADER_EXPR + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_index_set(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
        value: &HirExpr,
    ) -> CompileResult<MirOperand> {
        let obj_operand = self.lower_expr(object)?;
        let idx_operand = self.lower_expr(index)?;
        let val_operand = self.lower_expr(value)?;
        let unused = self.fresh_reg();
        self.emit(MirInstr::CallDirect(
            unused,
            "__bs_index_set".to_string(),
            vec![obj_operand, idx_operand, val_operand.clone()],
        ));
        Ok(val_operand)
    }
}
"""

expr_files["spread.rs"] = HEADER_EXPR + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr_spread(&self) -> CompileResult<MirOperand> {
        Err(CompileError::Lowering {
            message: "Spread expression outside array/object literals is unsupported".into(),
        })
    }
}
"""

# ──────────────────────────────────────────────────────────────────────────────
# Expression mod.rs — the router
# ──────────────────────────────────────────────────────────────────────────────

expr_mod = """use diagnostics::CompileResult;
use hir::HirExpr;
use crate::types::*;
use super::LowerCtx;

mod lit;
mod var;
mod bin_op;
mod unary_op;
mod call;
mod member_call;
mod assign;
mod ternary;
mod delete_prop;
mod yield_expr;
mod await_expr;
mod seq;
mod global_ref;
mod json_tape;
mod new_expr;
mod member_get;
mod member_set;
mod method_call;
mod instance_of;
mod closure;
mod array_lit;
mod index_get;
mod index_set;
mod spread;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_expr(&mut self, expr: &HirExpr) -> CompileResult<MirOperand> {
        match expr {
            HirExpr::Lit(lit)                                   => self.lower_expr_lit(lit),
            HirExpr::Var(bid)                                   => self.lower_expr_var(bid),
            HirExpr::BinOp(op, left, right)                    => self.lower_expr_bin_op(op, left, right),
            HirExpr::UnaryOp(op, arg)                          => self.lower_expr_unary_op(op, arg),
            HirExpr::Call { callee, args }                      => self.lower_expr_call(callee, args),
            HirExpr::MemberCall { object, method, args }        => self.lower_expr_member_call(object, method, args),
            HirExpr::Assign { target, value }                  => self.lower_expr_assign(target, value),
            HirExpr::Ternary { cond, then_expr, else_expr }    => self.lower_expr_ternary(cond, then_expr, else_expr),
            HirExpr::DeleteProp { object, property }           => self.lower_expr_delete_prop(object, property),
            HirExpr::Yield { arg, delegate }                   => self.lower_expr_yield(arg, *delegate),
            HirExpr::Await(inner)                              => self.lower_expr_await(inner),
            HirExpr::Seq(exprs)                                => self.lower_expr_seq(exprs),
            HirExpr::GlobalRef(name)                           => self.lower_expr_global_ref(name),
            HirExpr::JsonTape(bytes)                           => self.lower_expr_json_tape(bytes),
            HirExpr::New { class_name, args }                  => self.lower_expr_new(class_name, args),
            HirExpr::MemberGet { object, property }            => self.lower_expr_member_get(object, property),
            HirExpr::MemberSet { object, property, value }     => self.lower_expr_member_set(object, property, value),
            HirExpr::MethodCall { object, method, args }       => self.lower_expr_method_call(object, method, args),
            HirExpr::InstanceOf { expr, class_name }           => self.lower_expr_instance_of(expr, class_name),
            HirExpr::Closure { func_id, captures }             => self.lower_expr_closure(func_id, captures),
            HirExpr::ArrayLit(elems)                           => self.lower_expr_array_lit(elems),
            HirExpr::IndexGet { object, index }                => self.lower_expr_index_get(object, index),
            HirExpr::IndexSet { object, index, value }         => self.lower_expr_index_set(object, index, value),
            HirExpr::Spread(_)                                 => self.lower_expr_spread(),
        }
    }
}
"""

# ──────────────────────────────────────────────────────────────────────────────
# Statement files
# ──────────────────────────────────────────────────────────────────────────────

stmt_files = {}

stmt_files["expr_stmt.rs"] = HEADER_STMT + """
use hir::HirExpr;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_expr(&mut self, e: &HirExpr) -> CompileResult<()> {
        self.lower_expr(e)?;
        Ok(())
    }
}
"""

stmt_files["let_stmt.rs"] = HEADER_STMT + """
use hir::{BindingId, HirExpr};

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_let(
        &mut self,
        binding: &BindingId,
        name: &str,
        init: &Option<HirExpr>,
    ) -> CompileResult<()> {
        let reg = self.fresh_reg();
        self.bind(*binding, reg);

        if self.capture_cells.contains(binding) {
            self.reg_shapes.insert(reg, "CaptureCell".to_string());
            self.emit(MirInstr::Alloc(reg, "CaptureCell".to_string()));
            let val = match init {
                Some(e) => self.lower_expr(e)?,
                None => MirOperand::ConstUndefined,
            };
            self.emit(MirInstr::StoreField(reg, 0, val.clone()));
            // Track class constructor bindings for static getter/setter interception
            if self.classes.contains_key(name) {
                self.emit(MirInstr::StoreGlobal(format!("__bs_class_val_{}", name), val));
            }
        } else {
            let val = match init {
                Some(e) => self.lower_expr(e)?,
                None => MirOperand::ConstUndefined,
            };
            if let MirOperand::Reg(src_reg) = &val {
                if let Some(shape) = self.reg_shapes.get(&src_reg).cloned() {
                    self.reg_shapes.insert(reg, shape);
                }
            }
            self.emit(MirInstr::Move(reg, val));
            // Track class constructor bindings for static getter/setter interception
            if self.classes.contains_key(name) {
                self.class_constructors.insert(reg, name.to_string());
                self.emit(MirInstr::StoreGlobal(format!("__bs_class_val_{}", name), MirOperand::Reg(reg)));
            }
        }
        Ok(())
    }
}
"""

stmt_files["assign_stmt.rs"] = HEADER_STMT + """
use hir::{BindingId, HirExpr};

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_assign(
        &mut self,
        target: &BindingId,
        value: &HirExpr,
    ) -> CompileResult<()> {
        let val = self.lower_expr(value)?;
        if let Some(&reg) = self.bindings.get(target) {
            if self.capture_cells.contains(target) {
                self.emit(MirInstr::StoreField(reg, 0, val));
            } else {
                if let MirOperand::Reg(src_reg) = &val {
                    if let Some(shape) = self.reg_shapes.get(&src_reg).cloned() {
                        self.reg_shapes.insert(reg, shape);
                    }
                }
                self.emit(MirInstr::Move(reg, val));
            }
        }
        Ok(())
    }
}
"""

stmt_files["if_stmt.rs"] = HEADER_STMT + """
use hir::HirExpr;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_if(
        &mut self,
        cond: &HirExpr,
        then_body: &[HirStmt],
        else_body: &Option<Vec<HirStmt>>,
    ) -> CompileResult<()> {
        let cv = self.lower_expr(cond)?;
        let then_bb = self.fresh_block();
        let else_bb = self.fresh_block();
        let merge_bb = self.fresh_block();

        self.emit(MirInstr::Branch(cv, then_bb, else_bb));

        self.switch_to(then_bb);
        self.lower_stmts(then_body)?;
        if !self.current_block_terminated() {
            self.emit(MirInstr::Jump(merge_bb));
        }

        self.switch_to(else_bb);
        if let Some(els) = else_body {
            self.lower_stmts(els)?;
        }
        if !self.current_block_terminated() {
            self.emit(MirInstr::Jump(merge_bb));
        }

        self.switch_to(merge_bb);
        Ok(())
    }
}
"""

stmt_files["while_stmt.rs"] = HEADER_STMT + """
use hir::HirExpr;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_while(
        &mut self,
        cond: &HirExpr,
        body: &[HirStmt],
    ) -> CompileResult<()> {
        let cond_bb = self.fresh_block();
        let body_bb = self.fresh_block();
        let exit_bb = self.fresh_block();

        self.emit(MirInstr::Jump(cond_bb));

        self.switch_to(cond_bb);
        let cv = self.lower_expr(cond)?;
        self.emit(MirInstr::Branch(cv, body_bb, exit_bb));

        let label = self.next_loop_label.take();
        self.loop_stack.push(LoopStackFrame {
            label,
            continue_target: Some(cond_bb),
            break_target: exit_bb,
        });
        self.switch_to(body_bb);
        self.lower_stmts(body)?;
        if !self.current_block_terminated() {
            self.emit(MirInstr::Jump(cond_bb));
        }
        self.loop_stack.pop();

        self.switch_to(exit_bb);
        Ok(())
    }
}
"""

stmt_files["do_while.rs"] = HEADER_STMT + """
use hir::HirExpr;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_do_while(
        &mut self,
        body: &[HirStmt],
        cond: &HirExpr,
    ) -> CompileResult<()> {
        let body_bb = self.fresh_block();
        let cond_bb = self.fresh_block();
        let exit_bb = self.fresh_block();

        self.emit(MirInstr::Jump(body_bb));

        let label = self.next_loop_label.take();
        self.loop_stack.push(LoopStackFrame {
            label,
            continue_target: Some(cond_bb),
            break_target: exit_bb,
        });
        self.switch_to(body_bb);
        self.lower_stmts(body)?;
        if !self.current_block_terminated() {
            self.emit(MirInstr::Jump(cond_bb));
        }
        self.loop_stack.pop();

        self.switch_to(cond_bb);
        let cv = self.lower_expr(cond)?;
        self.emit(MirInstr::Branch(cv, body_bb, exit_bb));

        self.switch_to(exit_bb);
        Ok(())
    }
}
"""

stmt_files["for_stmt.rs"] = HEADER_STMT + """
use hir::HirExpr;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_for(
        &mut self,
        init: &Option<Box<HirStmt>>,
        cond: &Option<HirExpr>,
        update: &Option<HirExpr>,
        body: &[HirStmt],
    ) -> CompileResult<()> {
        if let Some(i) = init {
            self.lower_stmt(i)?;
        }

        let cond_bb = self.fresh_block();
        let body_bb = self.fresh_block();
        let update_bb = self.fresh_block();
        let exit_bb = self.fresh_block();

        self.emit(MirInstr::Jump(cond_bb));

        self.switch_to(cond_bb);
        if let Some(c) = cond {
            let cv = self.lower_expr(c)?;
            self.emit(MirInstr::Branch(cv, body_bb, exit_bb));
        } else {
            self.emit(MirInstr::Jump(body_bb));
        }

        let label = self.next_loop_label.take();
        self.loop_stack.push(LoopStackFrame {
            label,
            continue_target: Some(update_bb),
            break_target: exit_bb,
        });
        self.switch_to(body_bb);
        self.lower_stmts(body)?;
        if !self.current_block_terminated() {
            self.emit(MirInstr::Jump(update_bb));
        }
        self.loop_stack.pop();

        self.switch_to(update_bb);
        if let Some(u) = update {
            self.lower_expr(u)?;
        }
        self.emit(MirInstr::Jump(cond_bb));

        self.switch_to(exit_bb);
        Ok(())
    }
}
"""

stmt_files["for_of.rs"] = HEADER_STMT + """
use hir::HirExpr;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_for_of(
        &mut self,
        left: &HirStmt,
        right: &HirExpr,
        body: &[HirStmt],
        is_await: bool,
    ) -> CompileResult<()> {
        let iter_reg = self.fresh_reg();
        let iter_val = self.lower_expr(right)?;
        self.emit(MirInstr::Move(iter_reg, iter_val));

        let cond_bb = self.fresh_block();
        let body_bb = self.fresh_block();
        let exit_bb = self.fresh_block();

        self.emit(MirInstr::Jump(cond_bb));
        self.switch_to(cond_bb);

        // Call generator.next()
        let next_val_reg = self.fresh_reg();
        self.emit(MirInstr::CallBuiltin(
            next_val_reg,
            BuiltinFn::GeneratorNext,
            vec![MirOperand::Reg(iter_reg), MirOperand::ConstUndefined],
        ));

        let resolved_val_reg = if is_await {
            let yield_idx = self.num_yield_points;
            self.num_yield_points += 1;

            let mut saves = Vec::new();
            for r in 0..self.next_reg {
                saves.push(r);
            }
            self.yield_saves.push(saves);

            self.emit(MirInstr::Suspend(yield_idx, MirOperand::Reg(next_val_reg)));
            let dest = self.fresh_reg();
            self.emit(MirInstr::Resume(dest, yield_idx));
            dest
        } else {
            next_val_reg
        };

        // Check if iterator is done
        let is_done_reg = self.fresh_reg();
        self.emit(MirInstr::CallBuiltin(
            is_done_reg,
            BuiltinFn::GeneratorIsDone,
            vec![MirOperand::Reg(iter_reg)],
        ));
        self.emit(MirInstr::Branch(MirOperand::Reg(is_done_reg), exit_bb, body_bb));

        self.switch_to(body_bb);
        let label = self.next_loop_label.take();
        self.loop_stack.push(LoopStackFrame {
            label,
            continue_target: Some(cond_bb),
            break_target: exit_bb,
        });

        // Declare the loop variable if it's a Let, or evaluate the assignee
        self.lower_stmt(left)?;

        // Assign to left
        match left {
            HirStmt::Expr(HirExpr::Assign { target, .. }) => {
                let reg = self.bindings[target];
                if self.capture_cells.contains(target) {
                    self.emit(MirInstr::StoreField(reg, 0, MirOperand::Reg(resolved_val_reg)));
                } else {
                    self.emit(MirInstr::Move(reg, MirOperand::Reg(resolved_val_reg)));
                }
            }
            HirStmt::Let { binding, name: _, init: _ } => {
                let reg = self.bindings[binding];
                if self.capture_cells.contains(binding) {
                    self.emit(MirInstr::StoreField(reg, 0, MirOperand::Reg(resolved_val_reg)));
                } else {
                    self.emit(MirInstr::Move(reg, MirOperand::Reg(resolved_val_reg)));
                }
            }
            _ => unreachable!("for..of left is neither assignment nor let"),
        }

        self.lower_stmts(body)?;

        if !self.current_block_terminated() {
            self.emit(MirInstr::Jump(cond_bb));
        }

        self.loop_stack.pop();
        self.switch_to(exit_bb);

        Ok(())
    }
}
"""

stmt_files["return_stmt.rs"] = HEADER_STMT + """
use hir::HirExpr;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_return(&mut self, val: &Option<HirExpr>) -> CompileResult<()> {
        let v = match val {
            Some(e) => Some(self.lower_expr(e)?),
            None => None,
        };
        self.emit(MirInstr::Return(v));
        Ok(())
    }
}
"""

stmt_files["break_stmt.rs"] = HEADER_STMT + """
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
"""

stmt_files["continue_stmt.rs"] = HEADER_STMT + """
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
"""

stmt_files["switch.rs"] = HEADER_STMT + """
use hir::HirExpr;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_switch(
        &mut self,
        discriminant: &HirExpr,
        cases: &[hir::SwitchCase],
    ) -> CompileResult<()> {
        let disc_operand = self.lower_expr(discriminant)?;
        let exit_bb = self.fresh_block();

        let mut body_bbs = Vec::new();
        for _ in cases {
            body_bbs.push(self.fresh_block());
        }

        self.loop_stack.push(LoopStackFrame {
            label: None,
            continue_target: None,
            break_target: exit_bb,
        });

        let mut current_test_bb = self.current;
        let mut default_idx = None;

        for (i, case) in cases.iter().enumerate() {
            if let Some(test_expr) = &case.test {
                let next_test_bb = self.fresh_block();
                self.switch_to(current_test_bb);

                let test_operand = self.lower_expr(test_expr)?;
                let eq_reg = self.fresh_reg();
                self.emit(MirInstr::StrictEq(eq_reg, disc_operand.clone(), test_operand));
                self.emit(MirInstr::Branch(MirOperand::Reg(eq_reg), body_bbs[i], next_test_bb));

                current_test_bb = next_test_bb;
            } else {
                default_idx = Some(i);
            }
        }

        self.switch_to(current_test_bb);
        if let Some(def_i) = default_idx {
            self.emit(MirInstr::Jump(body_bbs[def_i]));
        } else {
            self.emit(MirInstr::Jump(exit_bb));
        }

        for (i, case) in cases.iter().enumerate() {
            self.switch_to(body_bbs[i]);
            self.lower_stmts(&case.consequent)?;

            if !self.current_block_terminated() {
                let next_bb = if i + 1 < cases.len() {
                    body_bbs[i + 1]
                } else {
                    exit_bb
                };
                self.emit(MirInstr::Jump(next_bb));
            }
        }

        self.loop_stack.pop();
        self.switch_to(exit_bb);
        Ok(())
    }
}
"""

stmt_files["block.rs"] = HEADER_STMT + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_block(&mut self, stmts: &[HirStmt]) -> CompileResult<()> {
        self.lower_stmts(stmts)
    }
}
"""

stmt_files["labeled.rs"] = HEADER_STMT + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_labeled(
        &mut self,
        label: &str,
        body: &HirStmt,
    ) -> CompileResult<()> {
        let old_label = self.next_loop_label.take();
        self.next_loop_label = Some(label.to_string());

        let is_loop = matches!(
            body,
            HirStmt::While { .. }
                | HirStmt::DoWhile { .. }
                | HirStmt::For { .. }
                | HirStmt::ForOf { .. }
        );

        if is_loop {
            self.lower_stmt(body)?;
        } else {
            let exit_bb = self.fresh_block();
            self.loop_stack.push(LoopStackFrame {
                label: Some(label.to_string()),
                continue_target: None,
                break_target: exit_bb,
            });
            self.lower_stmt(body)?;
            self.loop_stack.pop();
            if !self.current_block_terminated() {
                self.emit(MirInstr::Jump(exit_bb));
            }
            self.switch_to(exit_bb);
        }
        self.next_loop_label = old_label;
        Ok(())
    }
}
"""

stmt_files["func_decl.rs"] = HEADER_STMT + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_func_decl(&self) -> CompileResult<()> {
        Ok(()) // handled at module level
    }
}
"""

stmt_files["throw.rs"] = HEADER_STMT + """
use hir::HirExpr;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_throw(&mut self, expr: &HirExpr) -> CompileResult<()> {
        let val = self.lower_expr(expr)?;
        self.emit(MirInstr::Throw(val));
        Ok(())
    }
}
"""

stmt_files["try_stmt.rs"] = HEADER_STMT + """
use hir::{BindingId, HirExpr};

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmt_try(
        &mut self,
        body: &[HirStmt],
        catch_param: &Option<(BindingId, String)>,
        catch_body: &[HirStmt],
        finally_body: &Option<Vec<HirStmt>>,
    ) -> CompileResult<()> {
        if let Some(fin_stmts) = finally_body {
            let finally_caught_reg = self.fresh_reg();
            let finally_err_reg = self.fresh_reg();
            self.emit(MirInstr::Move(finally_caught_reg, MirOperand::ConstBool(false)));

            let jmp_buf_reg = self.fresh_reg();
            self.emit(MirInstr::TryEnter(jmp_buf_reg));

            let setjmp_result = self.fresh_reg();
            self.emit(MirInstr::SetJmp(setjmp_result, jmp_buf_reg));

            let try_body_bb = self.fresh_block();
            let catch_bb = self.fresh_block();
            let finally_bb = self.fresh_block();

            self.emit(MirInstr::Branch(MirOperand::Reg(setjmp_result), catch_bb, try_body_bb));

            self.switch_to(try_body_bb);

            let has_catch = catch_param.is_some() || !catch_body.is_empty();
            if has_catch {
                self.lower_stmt_try(body, catch_param, catch_body, &None)?;
            } else {
                self.lower_stmts(body)?;
            }

            if !self.current_block_terminated() {
                self.emit(MirInstr::TryExit);
                self.emit(MirInstr::Jump(finally_bb));
            }

            self.switch_to(catch_bb);
            self.emit(MirInstr::CallDirect(
                finally_err_reg,
                "__bs_get_and_clear_exception".to_string(),
                vec![],
            ));
            self.emit(MirInstr::Move(finally_caught_reg, MirOperand::ConstBool(true)));
            self.emit(MirInstr::Jump(finally_bb));

            self.switch_to(finally_bb);
            self.lower_stmts(fin_stmts)?;

            let rethrow_bb = self.fresh_block();
            let merge_bb = self.fresh_block();
            self.emit(MirInstr::Branch(MirOperand::Reg(finally_caught_reg), rethrow_bb, merge_bb));

            self.switch_to(rethrow_bb);
            self.emit(MirInstr::Throw(MirOperand::Reg(finally_err_reg)));

            self.switch_to(merge_bb);
        } else {
            let jmp_buf_reg = self.fresh_reg();
            self.emit(MirInstr::TryEnter(jmp_buf_reg));

            let setjmp_result = self.fresh_reg();
            self.emit(MirInstr::SetJmp(setjmp_result, jmp_buf_reg));

            let try_body_bb = self.fresh_block();
            let catch_bb = self.fresh_block();
            let merge_bb = self.fresh_block();

            self.emit(MirInstr::Branch(MirOperand::Reg(setjmp_result), catch_bb, try_body_bb));

            self.switch_to(try_body_bb);
            self.lower_stmts(body)?;
            if !self.current_block_terminated() {
                self.emit(MirInstr::TryExit);
                self.emit(MirInstr::Jump(merge_bb));
            }

            self.switch_to(catch_bb);
            if let Some((bid, _name)) = catch_param {
                let exc_reg = self.fresh_reg();
                self.bind(*bid, exc_reg);
                self.emit(MirInstr::CallDirect(
                    exc_reg,
                    "__bs_get_and_clear_exception".to_string(),
                    vec![],
                ));
            } else {
                let unused = self.fresh_reg();
                self.emit(MirInstr::CallDirect(
                    unused,
                    "__bs_get_and_clear_exception".to_string(),
                    vec![],
                ));
            }
            self.lower_stmts(catch_body)?;
            if !self.current_block_terminated() {
                self.emit(MirInstr::Jump(merge_bb));
            }

            self.switch_to(merge_bb);
        }
        Ok(())
    }
}
"""

# ──────────────────────────────────────────────────────────────────────────────
# Statement mod.rs — the router
# ──────────────────────────────────────────────────────────────────────────────

stmt_mod = """use diagnostics::CompileResult;
use hir::HirStmt;
use crate::types::*;
use super::{LowerCtx, LoopStackFrame};

mod expr_stmt;
mod let_stmt;
mod assign_stmt;
mod if_stmt;
mod while_stmt;
mod do_while;
mod for_stmt;
mod for_of;
mod return_stmt;
mod break_stmt;
mod continue_stmt;
mod switch;
mod block;
mod labeled;
mod func_decl;
mod throw;
mod try_stmt;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_stmts(&mut self, stmts: &[HirStmt]) -> CompileResult<()> {
        for s in stmts {
            self.lower_stmt(s)?;
            // Stop emitting after a terminator (throw, return, break, continue)
            if self.current_block_terminated() {
                break;
            }
        }
        Ok(())
    }

    pub(super) fn lower_stmt(&mut self, stmt: &HirStmt) -> CompileResult<()> {
        match stmt {
            HirStmt::Expr(e)                                                     => self.lower_stmt_expr(e),
            HirStmt::Let { binding, name, init }                                 => self.lower_stmt_let(binding, name, init),
            HirStmt::Assign { target, value }                                    => self.lower_stmt_assign(target, value),
            HirStmt::If { cond, then_body, else_body }                           => self.lower_stmt_if(cond, then_body, else_body),
            HirStmt::While { cond, body }                                        => self.lower_stmt_while(cond, body),
            HirStmt::DoWhile { body, cond }                                      => self.lower_stmt_do_while(body, cond),
            HirStmt::For { init, cond, update, body }                            => self.lower_stmt_for(init, cond, update, body),
            HirStmt::ForOf { left, right, body, is_await }                       => self.lower_stmt_for_of(left, right, body, *is_await),
            HirStmt::Return(val)                                                  => self.lower_stmt_return(val),
            HirStmt::Break(label)                                                 => self.lower_stmt_break(label),
            HirStmt::Continue(label)                                              => self.lower_stmt_continue(label),
            HirStmt::Switch { discriminant, cases }                              => self.lower_stmt_switch(discriminant, cases),
            HirStmt::Block(stmts)                                                => self.lower_stmt_block(stmts),
            HirStmt::Labeled { label, body }                                     => self.lower_stmt_labeled(label, body),
            HirStmt::FuncDecl { .. }                                             => self.lower_stmt_func_decl(),
            HirStmt::Throw(expr)                                                  => self.lower_stmt_throw(expr),
            HirStmt::Try { body, catch_param, catch_body, finally_body }         => self.lower_stmt_try(body, catch_param, catch_body, finally_body),
        }
    }
}
"""

# ──────────────────────────────────────────────────────────────────────────────
# Builtin files
# ──────────────────────────────────────────────────────────────────────────────

builtin_files = {}

builtin_files["console.rs"] = HEADER_BUILTIN + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_builtin_console(
        &mut self,
        method: &str,
        mir_args: Vec<MirOperand>,
        dest: MirReg,
    ) -> CompileResult<bool> {
        match method {
            "log" | "error" => {
                self.emit(MirInstr::CallBuiltin(dest, BuiltinFn::ConsoleLog, mir_args));
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
"""

builtin_files["promise.rs"] = HEADER_BUILTIN + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_builtin_promise(
        &mut self,
        method: &str,
        mir_args: Vec<MirOperand>,
        dest: MirReg,
    ) -> CompileResult<bool> {
        match method {
            "all_2" => {
                self.emit(MirInstr::CallBuiltin(dest, BuiltinFn::PromiseAll2, mir_args));
                Ok(true)
            }
            "race_2" => {
                self.emit(MirInstr::CallBuiltin(dest, BuiltinFn::PromiseRace2, mir_args));
                Ok(true)
            }
            "resolve" => {
                self.emit(MirInstr::CallDirect(
                    dest,
                    "__bs_promise_static_resolve".to_string(),
                    mir_args,
                ));
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
"""

builtin_files["number.rs"] = HEADER_BUILTIN + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_builtin_number(
        &mut self,
        method: &str,
        mir_args: Vec<MirOperand>,
        dest: MirReg,
    ) -> CompileResult<bool> {
        match method {
            "isInteger" => {
                self.emit(MirInstr::CallDirect(dest, "__bs_number_isInteger".to_string(), mir_args));
                Ok(true)
            }
            "isFinite" => {
                self.emit(MirInstr::CallDirect(dest, "__bs_isFinite".to_string(), mir_args));
                Ok(true)
            }
            "isNaN" => {
                self.emit(MirInstr::CallDirect(dest, "__bs_isNaN".to_string(), mir_args));
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
"""

builtin_files["object.rs"] = HEADER_BUILTIN + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_builtin_object(
        &mut self,
        method: &str,
        mir_args: Vec<MirOperand>,
        dest: MirReg,
    ) -> CompileResult<bool> {
        match method {
            "keys" | "values" | "entries" | "assign" | "create" | "getPrototypeOf" | "fromEntries" => {
                self.emit(MirInstr::CallDirect(
                    dest,
                    format!("__bs_object_{}", method),
                    mir_args,
                ));
                Ok(true)
            }
            _ => Err(CompileError::Lowering {
                message: format!("Object.{}() not supported", method),
            }),
        }
    }
}
"""

builtin_files["string.rs"] = HEADER_BUILTIN + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_builtin_string(
        &mut self,
        method: &str,
        mir_args: Vec<MirOperand>,
        dest: MirReg,
    ) -> CompileResult<bool> {
        match method {
            "fromCharCode" | "fromCodePoint" => {
                self.emit(MirInstr::CallDirect(
                    dest,
                    format!("__bs_string_{}", method),
                    mir_args,
                ));
                Ok(true)
            }
            _ => Err(CompileError::Lowering {
                message: format!("String.{}() not supported", method),
            }),
        }
    }
}
"""

builtin_files["date.rs"] = HEADER_BUILTIN + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_builtin_date(
        &mut self,
        method: &str,
        mir_args: Vec<MirOperand>,
        dest: MirReg,
    ) -> CompileResult<bool> {
        match method {
            "now" => {
                self.emit(MirInstr::CallDirect(dest, "__bs_date_now".to_string(), mir_args));
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
"""

builtin_files["math.rs"] = HEADER_BUILTIN + """
impl<'a> LowerCtx<'a> {
    pub(super) fn lower_builtin_math(
        &mut self,
        method: &str,
        mir_args: Vec<MirOperand>,
        dest: MirReg,
    ) -> CompileResult<bool> {
        match method {
            "floor" | "ceil" | "round" | "abs" | "sqrt" | "pow" | "min" | "max"
            | "log" | "log2" | "sin" | "cos" | "tan" | "random" | "trunc" => {
                self.emit(MirInstr::CallDirect(
                    dest,
                    format!("__bs_math_{}", method),
                    mir_args,
                ));
                Ok(true)
            }
            _ => Err(CompileError::Lowering {
                message: format!("Math.{}() not supported", method),
            }),
        }
    }
}
"""

# ──────────────────────────────────────────────────────────────────────────────
# Builtin mod.rs — the router
# ──────────────────────────────────────────────────────────────────────────────

builtin_mod = """use diagnostics::CompileResult;
use crate::types::*;
use super::LowerCtx;

mod console;
mod promise;
mod number;
mod object;
mod string;
mod date;
mod math;

impl<'a> LowerCtx<'a> {
    pub(super) fn lower_builtin_member_call(
        &mut self,
        object: &str,
        method: &str,
        mir_args: Vec<MirOperand>,
        dest: MirReg,
    ) -> CompileResult<bool> {
        match object {
            "console" => self.lower_builtin_console(method, mir_args, dest),
            "Promise"  => self.lower_builtin_promise(method, mir_args, dest),
            "Number"   => self.lower_builtin_number(method, mir_args, dest),
            "Object"   => self.lower_builtin_object(method, mir_args, dest),
            "String"   => self.lower_builtin_string(method, mir_args, dest),
            "Date"     => self.lower_builtin_date(method, mir_args, dest),
            "Math"     => self.lower_builtin_math(method, mir_args, dest),
            _          => Ok(false),
        }
    }
}
"""

# ──────────────────────────────────────────────────────────────────────────────
# Write everything out
# ──────────────────────────────────────────────────────────────────────────────

def write(path, content):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        f.write(content)
    print(f"  wrote {path}")

# Expression sub-folder
for name, content in expr_files.items():
    write(f"{BASE}/expr/{name}", content)
write(f"{BASE}/expr/mod.rs", expr_mod)

# Statement sub-folder
for name, content in stmt_files.items():
    write(f"{BASE}/stmt/{name}", content)
write(f"{BASE}/stmt/mod.rs", stmt_mod)

# Builtin sub-folder
for name, content in builtin_files.items():
    write(f"{BASE}/builtins/{name}", content)
write(f"{BASE}/builtins/mod.rs", builtin_mod)

print("Done! Now remove the old flat files and update lower/mod.rs.")
