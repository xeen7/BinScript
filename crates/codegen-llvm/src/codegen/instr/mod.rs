#![allow(unused_imports)]
use inkwell::values::BasicMetadataValueEnum;
use inkwell::IntPredicate;
use inkwell::FloatPredicate;
use mir::types::*;
use mir::BuiltinFn;
use diagnostics::{CompileError, CompileResult};

use crate::codegen::LlvmCodegen;

pub mod arithmetic;
pub mod compare;
pub mod logical;
pub mod calls;
pub mod control_flow;
pub mod generators;
pub mod objects;
pub mod closures;
pub mod exceptions;
pub mod globals;
pub mod memory;
pub mod raii;

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
            MirInstr::CallDirect(..) | MirInstr::CallPure(..) => self.emit_instr_call_direct(instr)?,
            MirInstr::CallBuiltin(..) => self.emit_instr_call_builtin(instr)?,
            MirInstr::CallVTable(..) => self.emit_instr_call_vtable(instr)?,
            MirInstr::CallClosure(..) => self.emit_instr_call_closure(instr)?,
            MirInstr::AllocClosure(..) | MirInstr::AllocSharedClosure(..) | MirInstr::AllocOwnedClosure(..) => self.emit_instr_alloc_closure(instr)?,
            MirInstr::Branch(..) => self.emit_instr_branch(instr)?,
            MirInstr::Jump(..) => self.emit_instr_jump(instr)?,
            MirInstr::Return(..) => self.emit_instr_return_op(instr)?,
            MirInstr::Suspend(..) => self.emit_instr_suspend(instr)?,
            MirInstr::Resume(..) => self.emit_instr_resume(instr)?,
            MirInstr::TryEnter { .. } => self.emit_instr_try_enter(instr)?,
            MirInstr::TryExit => self.emit_instr_try_exit(instr)?,
            MirInstr::Throw(..) => self.emit_instr_throw(instr)?,
            MirInstr::Rethrow(..) => self.emit_instr_rethrow(instr)?,
            MirInstr::LandingPad { .. } => self.emit_instr_landing_pad(instr)?,
            MirInstr::ExtractException { .. } => self.emit_instr_extract_exception(instr)?,
            MirInstr::LoadGlobal(..) => self.emit_instr_load_global(instr)?,
            MirInstr::StoreGlobal(..) => self.emit_instr_store_global(instr)?,
            MirInstr::AllocShared(..) | MirInstr::AllocAcyclic(..) | MirInstr::AllocSharedAcyclic(..) | MirInstr::AllocOwned(..) | MirInstr::AllocStack(..) | MirInstr::AllocArena(..) => self.emit_instr_alloc_object(instr)?,
            MirInstr::ArenaCreate(..) => self.emit_instr_arena_create(instr)?,
            MirInstr::ArenaDestroy(..) => self.emit_instr_arena_destroy(instr)?,
            MirInstr::RcInc(..) => self.emit_instr_rc_inc(instr)?,
            MirInstr::RcDec(..) => self.emit_instr_rc_dec(instr)?,
            MirInstr::RcIncDeferred(..) => self.emit_instr_rc_inc_deferred(instr)?,
            MirInstr::RcDecDeferred(..) => self.emit_instr_rc_dec_deferred(instr)?,
            MirInstr::FlushRcDelta => self.emit_instr_flush_rc_delta()?,
            MirInstr::Drop(..) => self.emit_instr_drop(instr)?,
            MirInstr::DropStack(..) => self.emit_instr_drop_stack(instr)?,
            MirInstr::CallDropFnOnly(..) => self.emit_instr_call_drop_fn_only(instr)?,
            MirInstr::Borrow(..) | MirInstr::BorrowMut(..) => self.emit_instr_borrow(instr)?,
            MirInstr::StoreSharedField(..) => self.emit_instr_store_shared_field(instr)?,
            MirInstr::ScopeGuardPush { scope_id, reg, release_fn } => self.emit_instr_scope_guard_push(*scope_id, *reg, release_fn)?,
            MirInstr::ScopeGuardCancel { scope_id, reg } => self.emit_instr_scope_guard_cancel(*scope_id, *reg)?,
            MirInstr::ScopeGuardFlushTo { current_scope: _, target_scope } => self.emit_instr_scope_guard_flush_to(*target_scope)?,
            MirInstr::ForceOwnedTag(..) => self.emit_instr_force_owned_tag(instr)?,
        }
        Ok(())
    }
}
