#![allow(unused_imports)]
use inkwell::values::BasicMetadataValueEnum;
use inkwell::IntPredicate;
use inkwell::FloatPredicate;
use mir::types::*;
use mir::BuiltinFn;
use diagnostics::{CompileError, CompileResult};

use crate::codegen::LlvmCodegen;

mod arithmetic;
mod compare;
mod logical;
mod calls;
mod control_flow;
mod generators;
mod objects;
mod closures;
mod exceptions;
mod globals;
mod memory;

impl<'ctx> LlvmCodegen<'ctx> {
    pub(super) fn emit_instr(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::Move(..) => self.emit_instr_move_reg(instr)?,
            MirInstr::Add(..) => self.emit_instr_add(instr)?,
            MirInstr::Sub(..) => self.emit_instr_subtract(instr)?,
            MirInstr::Mul(..) => self.emit_instr_multiply(instr)?,
            MirInstr::Div(..) => self.emit_instr_divide(instr)?,
            MirInstr::Mod(..) => self.emit_instr_modulo(instr)?,
            MirInstr::Exp(..) => self.emit_instr_exponent(instr)?,
            MirInstr::Plus(..) => self.emit_instr_unary_plus(instr)?,
            MirInstr::Neg(..) => self.emit_instr_unary_neg(instr)?,
            MirInstr::Lt(..) => self.emit_instr_less_than(instr)?,
            MirInstr::Le(..) => self.emit_instr_less_equal(instr)?,
            MirInstr::Gt(..) => self.emit_instr_greater_than(instr)?,
            MirInstr::Ge(..) => self.emit_instr_greater_equal(instr)?,
            MirInstr::Eq(..) | MirInstr::StrictEq(..) => self.emit_instr_equal(instr)?,
            MirInstr::Ne(..) | MirInstr::StrictNe(..) => self.emit_instr_not_equal(instr)?,
            MirInstr::Not(..) => self.emit_instr_logical_not(instr)?,
            MirInstr::BitAnd(..) | MirInstr::BitOr(..) | MirInstr::BitXor(..) |
            MirInstr::Shl(..) | MirInstr::Shr(..) | MirInstr::UShr(..) | MirInstr::BitNot(..) => self.emit_instr_bitwise(instr)?,
            MirInstr::In(..) => self.emit_instr_in_operator(instr)?,
            MirInstr::Alloc(..) => self.emit_instr_alloc_object(instr)?,
            MirInstr::LoadField(..) => self.emit_instr_load_field(instr)?,
            MirInstr::StoreField(..) => self.emit_instr_store_field(instr)?,
            MirInstr::LoadProp(..) => self.emit_instr_load_prop(instr)?,
            MirInstr::StoreProp(..) => self.emit_instr_store_prop(instr)?,
            MirInstr::DeleteProp(..) => self.emit_instr_delete_prop(instr)?,
            MirInstr::CallDirect(..) => self.emit_instr_call_direct(instr)?,
            MirInstr::CallBuiltin(..) => self.emit_instr_call_builtin(instr)?,
            MirInstr::CallVTable(..) => self.emit_instr_call_vtable(instr)?,
            MirInstr::CallClosure(..) => self.emit_instr_call_closure(instr)?,
            MirInstr::AllocClosure(..) => self.emit_instr_alloc_closure(instr)?,
            MirInstr::Branch(..) => self.emit_instr_branch(instr)?,
            MirInstr::Jump(..) => self.emit_instr_jump(instr)?,
            MirInstr::Return(..) => self.emit_instr_return_op(instr)?,
            MirInstr::Suspend(..) => self.emit_instr_suspend(instr)?,
            MirInstr::Resume(..) => self.emit_instr_resume(instr)?,
            MirInstr::TryEnter(..) => self.emit_instr_try_enter(instr)?,
            MirInstr::SetJmp(..) => self.emit_instr_set_jmp(instr)?,
            MirInstr::TryExit => self.emit_instr_try_exit(instr)?,
            MirInstr::Throw(..) => self.emit_instr_throw(instr)?,
            MirInstr::LoadGlobal(..) => self.emit_instr_load_global(instr)?,
            MirInstr::StoreGlobal(..) => self.emit_instr_store_global(instr)?,
        }
        Ok(())
    }
}
