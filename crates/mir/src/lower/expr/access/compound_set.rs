use diagnostics::CompileResult;
use hir::{HirExpr, BinOp as HBinOp};
use crate::types::*;
use crate::lower::LowerCtx;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr_compound_member_set(
        &mut self,
        object: &HirExpr,
        property: &str,
        op: &HBinOp,
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

        let prop_reg = self.fresh_reg();
        let mut load_emitted = false;
        
        let mut load_action = None;
        if let Some(shape) = self.reg_shapes.get(&obj_reg) {
            if self.has_getter(shape, property) {
                let getter_name = format!("__get_{}", property);
                if let Some(&method_idx) = self.method_indices.get(&getter_name) {
                    load_action = Some(Ok(method_idx));
                }
            }
            if load_action.is_none() {
                if let Some(index) = self.get_field_index(shape, property) {
                    load_action = Some(Err(index));
                }
            }
        }

        match load_action {
            Some(Ok(method_idx)) => {
                let mir_args = vec![MirOperand::Reg(obj_reg)];
                self.emit(MirInstr::CallVTable(prop_reg, obj_reg, method_idx, mir_args));
                load_emitted = true;
            }
            Some(Err(index)) => {
                self.emit(MirInstr::LoadField(prop_reg, obj_reg, index));
                load_emitted = true;
            }
            None => {}
        }

        if !load_emitted {
            let mut static_getter = false;
            if let Some(ctor_class) = self.class_constructors.get(&obj_reg).cloned() {
                if self.has_static_getter(&ctor_class, property) {
                    static_getter = true;
                }
            }
            if static_getter {
                let getter_prop = format!("__get_{}", property);
                let closure_reg = self.fresh_reg();
                self.emit(MirInstr::LoadProp(closure_reg, obj_reg, getter_prop));
                self.emit(MirInstr::CallClosure(prop_reg, closure_reg, vec![MirOperand::Reg(closure_reg)]));
                load_emitted = true;
            }
        }
        if !load_emitted {
            self.emit(MirInstr::LoadProp(prop_reg, obj_reg, property.to_string()));
        }

        let dest = self.fresh_reg();
        
        if matches!(op, HBinOp::And | HBinOp::Or | HBinOp::NullishCoalescing) {
            let eval_r_bb = self.fresh_block();
            let merge_bb = self.fresh_block();

            self.emit(MirInstr::Move(dest, MirOperand::Reg(prop_reg)));
            
            if let HBinOp::And = op {
                self.emit(MirInstr::Branch(MirOperand::Reg(prop_reg), eval_r_bb, merge_bb));
            } else if let HBinOp::Or = op {
                self.emit(MirInstr::Branch(MirOperand::Reg(prop_reg), merge_bb, eval_r_bb));
            } else {
                let cond_reg = self.fresh_reg();
                self.emit(MirInstr::CallDirect(cond_reg, "__bs_is_nullish".to_string(), vec![MirOperand::Reg(prop_reg)]));
                self.emit(MirInstr::Branch(MirOperand::Reg(cond_reg), eval_r_bb, merge_bb));
            }

            self.switch_to(eval_r_bb);
            let val = self.lower_expr(value)?;
            self.emit(MirInstr::Move(dest, val.clone()));
            
            self.emit_compound_store(obj_reg, property, val);

            self.emit(MirInstr::Jump(merge_bb));
            self.switch_to(merge_bb);
            return Ok(MirOperand::Reg(dest));
        }

        let val = self.lower_expr(value)?;
        let instr = match op {
            HBinOp::Add => MirInstr::Add(dest, MirOperand::Reg(prop_reg), val.clone()),
            HBinOp::Sub => MirInstr::Sub(dest, MirOperand::Reg(prop_reg), val.clone()),
            HBinOp::Mul => MirInstr::Mul(dest, MirOperand::Reg(prop_reg), val.clone()),
            HBinOp::Div => MirInstr::Div(dest, MirOperand::Reg(prop_reg), val.clone()),
            HBinOp::Mod => MirInstr::Mod(dest, MirOperand::Reg(prop_reg), val.clone()),
            HBinOp::Exp => MirInstr::Exp(dest, MirOperand::Reg(prop_reg), val.clone()),
            HBinOp::BitAnd => MirInstr::BitAnd(dest, MirOperand::Reg(prop_reg), val.clone()),
            HBinOp::BitOr => MirInstr::BitOr(dest, MirOperand::Reg(prop_reg), val.clone()),
            HBinOp::BitXor => MirInstr::BitXor(dest, MirOperand::Reg(prop_reg), val.clone()),
            HBinOp::Shl => MirInstr::Shl(dest, MirOperand::Reg(prop_reg), val.clone()),
            HBinOp::Shr => MirInstr::Shr(dest, MirOperand::Reg(prop_reg), val.clone()),
            HBinOp::UShr => MirInstr::UShr(dest, MirOperand::Reg(prop_reg), val.clone()),
            _ => unreachable!(),
        };
        self.emit(instr);
        
        let val_to_store = MirOperand::Reg(dest);
        self.emit_compound_store(obj_reg, property, val_to_store);

        Ok(MirOperand::Reg(dest))
    }

    fn emit_compound_store(&mut self, obj_reg: MirReg, property: &str, val: MirOperand) {
        let mut store_action = None;
        if let Some(shape) = self.reg_shapes.get(&obj_reg) {
            if self.has_setter(shape, property) {
                let setter_name = format!("__set_{}", property);
                if let Some(&method_idx) = self.method_indices.get(&setter_name) {
                    store_action = Some(Ok(method_idx));
                }
            }
            if store_action.is_none() {
                if let Some(index) = self.get_field_index(shape, property) {
                    store_action = Some(Err(index));
                }
            }
        }

        let mut store_emitted = false;
        match store_action {
            Some(Ok(method_idx)) => {
                let void_dest = self.fresh_reg();
                self.emit(MirInstr::CallVTable(void_dest, obj_reg, method_idx, vec![MirOperand::Reg(obj_reg), val.clone()]));
                store_emitted = true;
            }
            Some(Err(index)) => {
                self.emit(MirInstr::StoreField(obj_reg, index, val.clone()));
                store_emitted = true;
            }
            None => {}
        }

        if !store_emitted {
            let mut static_setter = false;
            if let Some(ctor_class) = self.class_constructors.get(&obj_reg).cloned() {
                if self.has_static_setter(&ctor_class, property) {
                    static_setter = true;
                }
            }
            if static_setter {
                let setter_prop = format!("__set_{}", property);
                let closure_reg = self.fresh_reg();
                self.emit(MirInstr::LoadProp(closure_reg, obj_reg, setter_prop));
                let void_dest = self.fresh_reg();
                self.emit(MirInstr::CallClosure(void_dest, closure_reg, vec![MirOperand::Reg(closure_reg), val.clone()]));
                store_emitted = true;
            }
        }
        if !store_emitted {
            self.emit(MirInstr::StoreProp(obj_reg, property.to_string(), val, false));
        }
    }

    pub(crate) fn lower_expr_compound_index_set(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
        op: &HBinOp,
        value: &HirExpr,
    ) -> CompileResult<MirOperand> {
        let obj_operand = self.lower_expr(object)?;
        let idx_operand = self.lower_expr(index)?;
        
        let prop_reg = self.fresh_reg();
        self.emit(MirInstr::CallDirect(
            prop_reg,
            "__bs_index_get".to_string(),
            vec![obj_operand.clone(), idx_operand.clone()],
        ));

        let dest = self.fresh_reg();

        if matches!(op, HBinOp::And | HBinOp::Or | HBinOp::NullishCoalescing) {
            let eval_r_bb = self.fresh_block();
            let merge_bb = self.fresh_block();

            self.emit(MirInstr::Move(dest, MirOperand::Reg(prop_reg)));
            
            if let HBinOp::And = op {
                self.emit(MirInstr::Branch(MirOperand::Reg(prop_reg), eval_r_bb, merge_bb));
            } else if let HBinOp::Or = op {
                self.emit(MirInstr::Branch(MirOperand::Reg(prop_reg), merge_bb, eval_r_bb));
            } else {
                let cond_reg = self.fresh_reg();
                self.emit(MirInstr::CallDirect(cond_reg, "__bs_is_nullish".to_string(), vec![MirOperand::Reg(prop_reg)]));
                self.emit(MirInstr::Branch(MirOperand::Reg(cond_reg), eval_r_bb, merge_bb));
            }

            self.switch_to(eval_r_bb);
            let val = self.lower_expr(value)?;
            self.emit(MirInstr::Move(dest, val.clone()));
            
            let unused = self.fresh_reg();
            self.emit(MirInstr::CallDirect(
                unused,
                "__bs_index_set".to_string(),
                vec![obj_operand.clone(), idx_operand.clone(), val],
            ));

            self.emit(MirInstr::Jump(merge_bb));
            self.switch_to(merge_bb);
            return Ok(MirOperand::Reg(dest));
        }

        let val = self.lower_expr(value)?;
        let instr = match op {
            HBinOp::Add => MirInstr::Add(dest, MirOperand::Reg(prop_reg), val.clone()),
            HBinOp::Sub => MirInstr::Sub(dest, MirOperand::Reg(prop_reg), val.clone()),
            HBinOp::Mul => MirInstr::Mul(dest, MirOperand::Reg(prop_reg), val.clone()),
            HBinOp::Div => MirInstr::Div(dest, MirOperand::Reg(prop_reg), val.clone()),
            HBinOp::Mod => MirInstr::Mod(dest, MirOperand::Reg(prop_reg), val.clone()),
            HBinOp::Exp => MirInstr::Exp(dest, MirOperand::Reg(prop_reg), val.clone()),
            HBinOp::BitAnd => MirInstr::BitAnd(dest, MirOperand::Reg(prop_reg), val.clone()),
            HBinOp::BitOr => MirInstr::BitOr(dest, MirOperand::Reg(prop_reg), val.clone()),
            HBinOp::BitXor => MirInstr::BitXor(dest, MirOperand::Reg(prop_reg), val.clone()),
            HBinOp::Shl => MirInstr::Shl(dest, MirOperand::Reg(prop_reg), val.clone()),
            HBinOp::Shr => MirInstr::Shr(dest, MirOperand::Reg(prop_reg), val.clone()),
            HBinOp::UShr => MirInstr::UShr(dest, MirOperand::Reg(prop_reg), val.clone()),
            _ => unreachable!(),
        };
        self.emit(instr);

        let unused = self.fresh_reg();
        self.emit(MirInstr::CallDirect(
            unused,
            "__bs_index_set".to_string(),
            vec![obj_operand, idx_operand, MirOperand::Reg(dest)],
        ));

        Ok(MirOperand::Reg(dest))
    }
}
