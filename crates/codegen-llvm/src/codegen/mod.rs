//! LLVM IR code generator.
//!
//! Translates MIR into LLVM IR via `inkwell`. Every JS value is represented
//! as a NaN-boxed `i64`.

use std::collections::HashMap;

use inkwell::basic_block::BasicBlock as LlvmBB;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::*;
use inkwell::values::*;
use inkwell::AddressSpace;
use inkwell::attributes::{Attribute, AttributeLoc};


use diagnostics::{CompileError, CompileResult};
use mir::types::*;

use crate::nan_box::NanBoxHelper;

mod func;
mod helpers;
mod instr;
mod drop_fn_gen;
mod trace_fn_gen;

// ===========================================================================
// Public interface
// ===========================================================================

pub struct LlvmCodegen<'ctx> {
    pub ctx: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,

    // Frequently-used types
    i64_ty: IntType<'ctx>,
    i32_ty: IntType<'ctx>,
    i8_ty: IntType<'ctx>,
    f64_ty: FloatType<'ctx>,
    ptr_ty: PointerType<'ctx>,
    void_ty: VoidType<'ctx>,

    /// NaN-boxing utilities.
    pub nan: NanBoxHelper<'ctx>,

    /// External + user function lookup table.
    funcs: HashMap<String, FunctionValue<'ctx>>,

    /// Mapping of HIR FuncId to lowered MIR function name.
    func_id_to_name: HashMap<hir::FuncId, String>,

    // ── per-function state (reset on each function emission) ───────────────
    regs: HashMap<MirReg, PointerValue<'ctx>>,
    bbs: HashMap<BlockId, LlvmBB<'ctx>>,
    str_counter: u32,
    str_cache: HashMap<String, GlobalValue<'ctx>>,

    // --- Stage 2 additions ---
    drop_fns: HashMap<String, FunctionValue<'ctx>>,
    trace_fns: HashMap<String, FunctionValue<'ctx>>,
    vtables: HashMap<String, GlobalValue<'ctx>>,
    classes: HashMap<String, hir::HirClass>,

    // Generator Codegen State
    gen_state_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    gen_sent_val: Option<inkwell::values::IntValue<'ctx>>,
    gen_state_ty: Option<inkwell::types::StructType<'ctx>>,
    resume_blocks: HashMap<u32, inkwell::basic_block::BasicBlock<'ctx>>,
    gen_num_args: u32,

    // Arena pointers for each active RegionId in the current function
    pub arena_ptrs: HashMap<u32, PointerValue<'ctx>>,

    // Exception handling
    pub exception_scope_stack: Vec<(u32, inkwell::basic_block::BasicBlock<'ctx>)>,
    
    pub deferred_clears: Vec<u32>,
    /// Maps catch_bb → compile-time RAII slot index at TryEnter.
    /// The catch LP only cleans up slots with index >= this value.
    pub catch_raii_indices: HashMap<inkwell::basic_block::BasicBlock<'ctx>, usize>,
    
    // RAII Frame Base for absolute scope depth (generators only)
    pub frame_base: Option<inkwell::values::IntValue<'ctx>>,

    // ── Zero-cost RAII cleanup landing pads ─────────────────────────────────
    /// Compile-time RAII slots: flag + value allocas per ScopeGuardPush site.
    pub raii_slots: Vec<RaiiSlot<'ctx>>,
    /// Maps MIR register → RAII slot index.
    pub raii_reg_to_slot: HashMap<MirReg, usize>,
    /// Cached cleanup-only landing pad (for calls outside try blocks).
    pub raii_cleanup_bb: Option<inkwell::basic_block::BasicBlock<'ctx>>,
    /// Counter tracking how many ScopeGuardPush instructions have been emitted.
    pub raii_push_counter: usize,

    pub verify_memory: bool,
}

/// A compile-time RAII slot with liveness flag and value storage.
/// Replaces the runtime GUARD_STACK for non-generator functions.
pub struct RaiiSlot<'ctx> {
    /// Stack-allocated i1 flag: true when the object is live.
    pub flag_ptr: PointerValue<'ctx>,
    /// Stack-allocated i64 slot storing the NaN-boxed object value.
    pub val_ptr: PointerValue<'ctx>,
    /// Name of the release function to call during cleanup.
    pub release_fn_name: String,
}

impl<'ctx> LlvmCodegen<'ctx> {
    pub fn new(ctx: &'ctx Context, module_name: &str, verify_memory: bool) -> Self {
        let module = ctx.create_module(module_name);
        let builder = ctx.create_builder();
        let i64_ty = ctx.i64_type();
        let i32_ty = ctx.i32_type();
        let i8_ty = ctx.i8_type();
        let f64_ty = ctx.f64_type();
        let ptr_ty = ctx.ptr_type(AddressSpace::default());
        let void_ty = ctx.void_type();
        let nan = NanBoxHelper::new(ctx, i64_ty, f64_ty, ctx.bool_type());

        Self {
            ctx,
            module,
            builder,
            i64_ty,
            i32_ty,
            i8_ty,
            f64_ty,
            ptr_ty,
            void_ty,
            nan,
            funcs: HashMap::new(),
            func_id_to_name: HashMap::new(),
            regs: HashMap::new(),
            bbs: HashMap::new(),
            str_counter: 0,
            str_cache: HashMap::new(),
            drop_fns: HashMap::new(),
            trace_fns: HashMap::new(),
            vtables: HashMap::new(),
            classes: HashMap::new(),
            gen_state_ptr: None,
            gen_sent_val: None,
            gen_state_ty: None,
            resume_blocks: HashMap::new(),
            gen_num_args: 0,
            arena_ptrs: HashMap::new(),
            exception_scope_stack: Vec::new(),
            catch_raii_indices: HashMap::new(),
            deferred_clears: Vec::new(),
            frame_base: None,
            raii_slots: Vec::new(),
            raii_reg_to_slot: HashMap::new(),
            raii_cleanup_bb: None,
            raii_push_counter: 0,
            verify_memory,
        }
    }

    // ── top-level entry ────────────────────────────────────────────────────

    pub fn emit_module(&mut self, mir: &MirModule) -> CompileResult<()> {
        self.declare_externals();
        self.emit_console_log_builtin();

        // Save class metadata
        self.classes = mir.classes.clone();
        self.func_id_to_name = mir.func_id_to_name.clone();

        // Forward-declare all user functions first.
        for f in &mir.functions {
            let params: Vec<BasicMetadataTypeEnum> =
                f.params.iter().map(|_| self.i64_ty.into()).collect();
            let ft = self.i64_ty.fn_type(&params, false);
            let fv = self.module.add_function(&f.name, ft, None);
            self.funcs.insert(f.name.clone(), fv);
        }

        // Generate RAII drop functions for classes before emitting vtables.
        self.generate_drop_fns()?;
        // Generate trace functions for cycle collector before emitting vtables.
        self.generate_trace_fns()?;

        // Generate static global vtables (references user functions, drop_fns and trace_fns).
        self.emit_vtables(mir)?;

        // Emit user functions.
        for f in &mir.functions {
            self.emit_function(f)?;
        }

        // Emit `main` wrapper.
        self.emit_main(&mir.main_body)?;
        Ok(())
    }

    // ── verification & output ──────────────────────────────────────────────

    pub fn verify(&self) -> CompileResult<()> {
        self.module.verify().map_err(|e| CompileError::Codegen {
            message: format!("LLVM verify: {}", e.to_string()),
        })
    }

    pub fn print_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }

    pub fn get_module(&self) -> &Module<'ctx> {
        &self.module
    }

    // ── external declarations ──────────────────────────────────────────────

    fn declare_externals(&mut self) {
        let add = |s: &mut Self, name: &str, ft: FunctionType<'ctx>| {
            let fv = s.module.add_function(name, ft, None);
            s.funcs.insert(name.into(), fv);
        };

        // int printf(const char*, ...)
        add(self, "printf", self.i32_ty.fn_type(&[self.ptr_ty.into()], true));
        // int putchar(int)
        add(self, "putchar", self.i32_ty.fn_type(&[self.i32_ty.into()], false));

        // ptr __bs_alloc(ptr vtable, i64 size)
        add(self, "__bs_alloc", self.i64_ty.fn_type(&[self.ptr_ty.into(), self.i64_ty.into()], false));

        // ptr __bs_alloc_acyclic(ptr vtable, i64 size)
        add(self, "__bs_alloc_acyclic", self.i64_ty.fn_type(&[self.ptr_ty.into(), self.i64_ty.into()], false));
        // ptr __bs_alloc_owned(ptr vtable_ptr, i64 size_in_bytes) -> i64
        add(self, "__bs_alloc_owned", self.i64_ty.fn_type(&[self.ptr_ty.into(), self.i64_ty.into()], false));
        // void __bs_free_owned(ptr obj_ptr)
        add(self, "__bs_free_owned", self.void_ty.fn_type(&[self.ptr_ty.into()], false));
        // void circ_inc(ptr header)
        add(self, "circ_inc", self.void_ty.fn_type(&[self.ptr_ty.into()], false));
        // void circ_dec(ptr header)
        add(self, "circ_dec", self.void_ty.fn_type(&[self.ptr_ty.into()], false));
        // void free(ptr mem)
        add(self, "free", self.void_ty.fn_type(&[self.ptr_ty.into()], false));
        
        // void __bs_cycle_collector_init()
        add(self, "__bs_cycle_collector_init", self.void_ty.fn_type(&[], false));

        // --- Verify Mode ---
        add(self, "__bs_set_verify_memory", self.void_ty.fn_type(&[self.i8_ty.into()], false));
        add(self, "__verify_load", self.void_ty.fn_type(&[self.ptr_ty.into()], false));
        add(self, "__verify_store", self.void_ty.fn_type(&[self.ptr_ty.into()], false));
        add(self, "__bs_verify_check_leaks", self.void_ty.fn_type(&[], false));
        
        // --- Arena Allocator ---
        // ptr arena_create(i64 initial_capacity)
        add(self, "arena_create", self.ptr_ty.fn_type(&[self.i64_ty.into()], false));
        // ptr arena_alloc(ptr arena, i64 size, i64 align)
        add(self, "arena_alloc", self.ptr_ty.fn_type(&[self.ptr_ty.into(), self.i64_ty.into(), self.i64_ty.into()], false));
        // void arena_register_dtor(ptr arena, ptr obj, ptr drop_fn)
        add(self, "arena_register_dtor", self.void_ty.fn_type(&[self.ptr_ty.into(), self.ptr_ty.into(), self.ptr_ty.into()], false));
        // void arena_reset(ptr arena)
        add(self, "arena_reset", self.void_ty.fn_type(&[self.ptr_ty.into()], false));
        // void arena_destroy(ptr arena)
        add(self, "arena_destroy", self.void_ty.fn_type(&[self.ptr_ty.into()], false));

        // --- RAII Scope Guards ---
        // void __bs_scope_guard_push(i32 scope_id, ptr obj_ptr, ptr release_fn)
        add(self, "__bs_scope_guard_push", self.void_ty.fn_type(&[self.i32_ty.into(), self.i64_ty.into(), self.ptr_ty.into()], false));
        // void __bs_scope_guard_cancel(i32 frame_base, i32 scope_id, i64 obj_ptr)
        add(self, "__bs_scope_guard_cancel", self.void_ty.fn_type(&[self.i32_ty.into(), self.i32_ty.into(), self.i64_ty.into()], false));
        // void __bs_scope_guard_flush_to(i32 frame_base, i32 target_scope_id)
        add(self, "__bs_scope_guard_flush_to", self.void_ty.fn_type(&[self.i32_ty.into(), self.i32_ty.into()], false));

        // i64 __bs_instanceof(i64 obj, i64 shape_id) -> i64
        add(self, "__bs_instanceof", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        // ptr __bs_alloc_closure(i64 size_in_bytes) -> i64
        add(self, "__bs_alloc_closure", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        // void circ_dec_tagged(i64)
        add(self, "circ_dec_tagged", self.void_ty.fn_type(&[self.i64_ty.into()], false));
        // void circ_inc_tagged(i64)
        add(self, "circ_inc_tagged", self.void_ty.fn_type(&[self.i64_ty.into()], false));
        // ptr __bs_alloc_generator(i64 size_in_bytes) -> i64
        add(self, "__bs_alloc_generator", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        // i64 __bs_generator_next(i64 gen_ptr, i64 sent_val) -> i64
        add(self, "__bs_generator_next", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        // i64 __bs_generator_is_done(i64 gen_ptr) -> i64
        add(self, "__bs_generator_is_done", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        // void __bs_drain_microtasks()
        add(self, "__bs_drain_microtasks", self.void_ty.fn_type(&[], false));
        // void __bs_drain_finalizers()
        add(self, "__bs_drain_finalizers", self.void_ty.fn_type(&[], false));
        // i64 __bs_promise_new()
        add(self, "__bs_promise_new", self.i64_ty.fn_type(&[], false));
        // void __bs_promise_resolve(i64 promise_tagged, i64 value_tagged)
        add(self, "__bs_promise_resolve", self.void_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        // i64 __bs_promise_then(i64 promise_tagged, i64 closure_tagged) -> i64 promise
        add(self, "__bs_promise_then", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        // i64 __bs_async_drive(i64 generator_tagged) -> i64 promise
        add(self, "__bs_async_drive", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        // i64 __bs_promise_static_resolve(i64 value_tagged) -> i64 promise
        add(self, "__bs_promise_static_resolve", self.i64_ty.fn_type(&[self.i64_ty.into()], false));

        // --- Promise Combinators ---
        // i64 __bs_promise_all_2(i64 p1_tagged, i64 p2_tagged) -> i64 promise
        add(self, "__bs_promise_all_2", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        // i64 __bs_promise_race_2(i64 p1_tagged, i64 p2_tagged) -> i64 promise
        add(self, "__bs_promise_race_2", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));

        // --- JSON Tape & Dynamic Properties ---
        // i64 __bs_json_parse_lazy(ptr json_str, i32 len)
        add(self, "__bs_json_parse_lazy", self.i64_ty.fn_type(&[self.ptr_ty.into(), self.i32_ty.into()], false));
        // i64 __bs_json_tape_get(i64 tape_tagged, ptr prop_str, i32 len)
        add(self, "__bs_json_tape_get", self.i64_ty.fn_type(&[self.i64_ty.into(), self.ptr_ty.into(), self.i32_ty.into()], false));
        // i64 __bs_prop_get(i64 obj_tagged, ptr prop_str, i32 len)
        add(self, "__bs_prop_get", self.i64_ty.fn_type(&[self.i64_ty.into(), self.ptr_ty.into(), self.i32_ty.into()], false));
        // i64 __bs_prop_set(i64 obj_tagged, ptr prop_str, i32 len, i64 val_tagged)
        add(self, "__bs_prop_set", self.i64_ty.fn_type(&[self.i64_ty.into(), self.ptr_ty.into(), self.i32_ty.into(), self.i64_ty.into()], false));
        // i64 __bs_prop_set_moved(i64 obj_tagged, ptr prop_str, i32 len, i64 val_tagged)
        add(self, "__bs_prop_set_moved", self.i64_ty.fn_type(&[self.i64_ty.into(), self.ptr_ty.into(), self.i32_ty.into(), self.i64_ty.into()], false));
        // i64 __bs_new_object() -> i64
        add(self, "__bs_new_object", self.i64_ty.fn_type(&[], false));

        // --- Stage 11: Array & String Dynamic Dispatch ---
        add(self, "__bs_index_get", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        add(self, "__bs_index_set", self.void_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into(), self.i64_ty.into()], false));
        add(self, "__bs_array_new", self.i64_ty.fn_type(&[], false));
        add(self, "__bs_array_push", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        add(self, "__bs_array_push_spread", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        add(self, "__bs_object_spread", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        add(self, "__bs_array_from", self.i64_ty.fn_type(&[self.ptr_ty.into(), self.i32_ty.into()], false));
        add(self, "__bs_call_apply", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into(), self.i64_ty.into()], false));
        add(self, "__bs_vcall_apply", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into(), self.i64_ty.into()], false));

        // Math Helpers
        let math_unary = self.i64_ty.fn_type(&[self.i64_ty.into()], false);
        let math_binary = self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false);
        add(self, "__bs_math_floor", math_unary);
        add(self, "__bs_math_ceil", math_unary);
        add(self, "__bs_math_round", math_unary);
        add(self, "__bs_math_abs", math_unary);
        add(self, "__bs_math_sqrt", math_unary);
        add(self, "__bs_math_pow", math_binary);
        add(self, "__bs_math_min", math_binary);
        add(self, "__bs_math_max", math_binary);
        add(self, "__bs_math_log", math_unary);
        add(self, "__bs_math_log2", math_unary);
        add(self, "__bs_math_sin", math_unary);
        add(self, "__bs_math_cos", math_unary);
        add(self, "__bs_math_tan", math_unary);
        add(self, "__bs_math_random", self.i64_ty.fn_type(&[], false));
        add(self, "__bs_math_trunc", math_unary);

        // Global Helpers
        add(self, "__bs_parseInt", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        add(self, "__bs_parseInt_1", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        add(self, "__bs_parseInt_2", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        add(self, "__bs_parseFloat", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        add(self, "__bs_isNaN", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        add(self, "__bs_isFinite", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        add(self, "__bs_number_isInteger", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        add(self, "__bs_number_isSafeInteger", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        add(self, "__bs_typeof", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        add(self, "__bs_json_stringify", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        add(self, "__bs_json_parse", self.i64_ty.fn_type(&[self.i64_ty.into()], false));

        // Builtin Constructors & Coercions
        let unary_c = self.i64_ty.fn_type(&[self.i64_ty.into()], false);
        add(self, "__bs_Object", unary_c);
        add(self, "__bs_Object_new", unary_c);
        add(self, "__bs_String", unary_c);
        add(self, "__bs_String_new", unary_c);
        add(self, "__bs_Number", unary_c);
        add(self, "__bs_Number_new", unary_c);
        add(self, "__bs_Boolean", unary_c);
        add(self, "__bs_Boolean_new", unary_c);
        add(self, "__bs_Date", unary_c);
        add(self, "__bs_Date_new", unary_c);
        add(self, "__bs_Array_new", unary_c);
        add(self, "__bs_RegExp_new", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));

        let nullary_c = self.i64_ty.fn_type(&[], false);
        add(self, "__bs_Object_new_0", nullary_c);
        add(self, "__bs_Object_new_1", unary_c);
        add(self, "__bs_String_new_0", nullary_c);
        add(self, "__bs_String_new_1", unary_c);
        add(self, "__bs_Number_new_0", nullary_c);
        add(self, "__bs_Number_new_1", unary_c);
        add(self, "__bs_Boolean_new_0", nullary_c);
        add(self, "__bs_Boolean_new_1", unary_c);
        add(self, "__bs_Date_new_0", nullary_c);
        add(self, "__bs_Date_new_1", unary_c);
        
        add(self, "__bs_Map_new_0", nullary_c);
        add(self, "__bs_Map_new_1", unary_c);
        add(self, "__bs_Set_new_0", nullary_c);
        add(self, "__bs_Set_new_1", unary_c);
        add(self, "__bs_WeakMap_new_0", nullary_c);
        add(self, "__bs_WeakMap_new_1", unary_c);
        add(self, "__bs_WeakSet_new_0", nullary_c);
        add(self, "__bs_WeakSet_new_1", unary_c);
        add(self, "__bs_WeakRef_new_1", unary_c);
        add(self, "__bs_FinalizationRegistry_new_1", unary_c);

        let date_n_c = self.i64_ty.fn_type(&[
            self.i64_ty.into(),
            self.i64_ty.into(),
            self.i64_ty.into(),
            self.i64_ty.into(),
            self.i64_ty.into(),
            self.i64_ty.into(),
            self.i64_ty.into(),
        ], false);
        add(self, "__bs_Date_new_n", date_n_c);

        // Object / String / Date Static Methods
        add(self, "__bs_object_keys", unary_c);
        add(self, "__bs_object_values", unary_c);
        add(self, "__bs_object_entries", unary_c);
        add(self, "__bs_object_assign", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        add(self, "__bs_object_create", unary_c);
        add(self, "__bs_object_getPrototypeOf", unary_c);
        add(self, "__bs_object_fromEntries", unary_c);
        add(self, "__bs_object_rest", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));

        // Global URI & globalThis Helpers
        add(self, "__bs_get_globalThis", self.i64_ty.fn_type(&[], false));
        add(self, "__bs_get_Symbol_global", self.i64_ty.fn_type(&[], false));
        add(self, "__bs_Symbol", unary_c);
        add(self, "__bs_Symbol_0", self.i64_ty.fn_type(&[], false));
        add(self, "__bs_Symbol_1", unary_c);
        add(self, "__bs_dynamic_import", unary_c);
        add(self, "__bs_encodeURI", unary_c);
        add(self, "__bs_decodeURI", unary_c);
        add(self, "__bs_encodeURIComponent", unary_c);
        add(self, "__bs_decodeURIComponent", unary_c);
        add(self, "__bs_URIError_new", unary_c);
        add(self, "__bs_string_fromCharCode", unary_c);
        add(self, "__bs_string_fromCodePoint", unary_c);
        add(self, "__bs_date_now", self.i64_ty.fn_type(&[], false));

        // Strict equality content helpers
        add(self, "__bs_strict_eq", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        add(self, "__bs_strict_ne", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));

        // JS + operator (numeric add OR string concatenation)
        add(self, "__bs_add", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));

        // Stage 15 operators
        add(self, "__bs_is_nullish", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        add(self, "__bs_exp", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        add(self, "__bs_in", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        add(self, "__bs_delete_prop", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));

        let dispatch_0 = self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false);
        let dispatch_1 = self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into(), self.i64_ty.into()], false);
        let dispatch_2 = self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into(), self.i64_ty.into(), self.i64_ty.into()], false);
        let dispatch_3 = self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into(), self.i64_ty.into(), self.i64_ty.into(), self.i64_ty.into()], false);

        add(self, "dummy_arr_0", dispatch_0);
        add(self, "dummy_arr_1", dispatch_1);
        add(self, "dummy_arr_2", dispatch_2);

        add(self, "__bs_call_push", dispatch_1);
        add(self, "__bs_call_pop", dispatch_0);
        add(self, "__bs_call_slice", dispatch_2);
        add(self, "__bs_call_indexOf", dispatch_1);
        add(self, "__bs_call_includes", dispatch_1);
        add(self, "__bs_call_next", dispatch_1);
        add(self, "__bs_call_join", dispatch_1);
        add(self, "__bs_call_reverse", dispatch_0);
        add(self, "__bs_call_concat", dispatch_1);
        add(self, "__bs_call_fill", dispatch_3);

        add(self, "__bs_call_forEach", dispatch_1);
        add(self, "__bs_call_map", dispatch_1);
        add(self, "__bs_call_filter", dispatch_1);
        add(self, "__bs_call_find", dispatch_1);
        add(self, "__bs_call_findIndex", dispatch_1);
        add(self, "__bs_call_every", dispatch_1);
        add(self, "__bs_call_some", dispatch_1);
        add(self, "__bs_call_reduce", dispatch_2);

        add(self, "__bs_call_charAt", dispatch_1);
        add(self, "__bs_call_charCodeAt", dispatch_1);
        add(self, "__bs_call_startsWith", dispatch_1);
        add(self, "__bs_call_endsWith", dispatch_1);
        add(self, "__bs_call_substring", dispatch_2);
        add(self, "__bs_call_split", dispatch_1);
        add(self, "__bs_call_trim", dispatch_0);
        add(self, "__bs_call_toUpperCase", dispatch_0);
        add(self, "__bs_call_toLowerCase", dispatch_0);
        add(self, "__bs_call_replace", dispatch_2);
        add(self, "__bs_call_repeat", dispatch_1);
        add(self, "__bs_call_padStart", dispatch_2);
        add(self, "__bs_call_padEnd", dispatch_2);

        add(self, "__bs_call_getTime", dispatch_0);
        add(self, "__bs_call_getFullYear", dispatch_0);
        add(self, "__bs_call_getMonth", dispatch_0);
        add(self, "__bs_call_getDate", dispatch_0);
        add(self, "__bs_call_getHours", dispatch_0);
        add(self, "__bs_call_getMinutes", dispatch_0);
        add(self, "__bs_call_getSeconds", dispatch_0);
        add(self, "__bs_call_toString", dispatch_1);
        add(self, "__bs_call_valueOf", dispatch_0);
        
        add(self, "__bs_call_toFixed", dispatch_1);
        add(self, "__bs_call_toPrecision", dispatch_1);

        // --- Node.js Compatibility ---
        // i64 __bs_fs_read_file_sync(i64 path_tagged) -> i64
        add(self, "__bs_fs_read_file_sync", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        // void __bs_fs_write_file_sync(i64 path_tagged, i64 data_tagged)
        add(self, "__bs_fs_write_file_sync", self.void_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        // i64 __bs_fs_exists_sync(i64 path_tagged) -> i64
        add(self, "__bs_fs_exists_sync", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        // i64 __bs_path_join(i64 a_tagged, i64 b_tagged) -> i64
        add(self, "__bs_path_join", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        // i64 __bs_path_resolve(i64 a_tagged, i64 b_tagged) -> i64
        add(self, "__bs_path_resolve", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        // i64 __bs_os_platform() -> i64
        add(self, "__bs_os_platform", self.i64_ty.fn_type(&[], false));
        // i64 __bs_os_arch() -> i64
        add(self, "__bs_os_arch", self.i64_ty.fn_type(&[], false));

        // --- Stage 12: Exception Handling ---
        // int _setjmp(ptr) -> i32
        let setjmp_ft = self.i32_ty.fn_type(&[self.ptr_ty.into()], false);
        let setjmp_fn = self.module.add_function("_setjmp", setjmp_ft, None);
        let returns_twice_kind = Attribute::get_named_enum_kind_id("returns_twice");
        let returns_twice_attr = self.ctx.create_enum_attribute(returns_twice_kind, 0);
        setjmp_fn.add_attribute(AttributeLoc::Function, returns_twice_attr);
        self.funcs.insert("_setjmp".into(), setjmp_fn);

        // void __bs_try_enter(ptr)
        add(self, "__bs_try_enter", self.void_ty.fn_type(&[self.ptr_ty.into()], false));
        // void __bs_try_exit()
        add(self, "__bs_try_exit", self.void_ty.fn_type(&[], false));
        // void __bs_throw(i64)
        add(self, "__bs_throw", self.void_ty.fn_type(&[self.i64_ty.into()], false));
        // i64 __bs_get_and_clear_exception()
        add(self, "__bs_get_and_clear_exception", self.i64_ty.fn_type(&[], false));

        // Error constructors
        add(self, "__bs_Error_new", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        add(self, "__bs_TypeError_new", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        add(self, "__bs_RangeError_new", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        add(self, "__bs_ReferenceError_new", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        add(self, "__bs_SyntaxError_new", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
    }

    // ── console.log built-in ───────────────────────────────────────────────

    fn emit_console_log_builtin(&mut self) {
        // __bs_console_log_1(value: i64) -> void
        let ft = self.void_ty.fn_type(&[self.i64_ty.into()], false);
        let func = self.module.add_function("__bs_console_log_1", ft, None);
        self.funcs.insert("__bs_console_log_1".into(), func);

        let entry = self.ctx.append_basic_block(func, "entry");
        let print_num = self.ctx.append_basic_block(func, "print_num");
        let check_tag = self.ctx.append_basic_block(func, "check_tag");
        let print_bool_t = self.ctx.append_basic_block(func, "print_true");
        let print_bool_f = self.ctx.append_basic_block(func, "print_false");
        let print_str = self.ctx.append_basic_block(func, "print_str");
        let print_obj = self.ctx.append_basic_block(func, "print_obj");
        let print_null = self.ctx.append_basic_block(func, "print_null");
        let print_undef = self.ctx.append_basic_block(func, "print_undef");
        let print_closure = self.ctx.append_basic_block(func, "print_closure");
        let print_symbol = self.ctx.append_basic_block(func, "print_symbol");
        let done = self.ctx.append_basic_block(func, "done");

        let val = func.get_first_param().unwrap().into_int_value();
        let printf_fn = self.funcs["printf"];
        let putchar_fn = self.funcs["putchar"];

        // ── entry: is it tagged? ───────────────────────────────────────────
        self.builder.position_at_end(entry);
        let shifted = self.builder
            .build_right_shift(val, self.i64_ty.const_int(48, false), false, "top16")
            .unwrap();
        let tag_min = self.i64_ty.const_int(0xFFF1, false);
        let is_tagged = self.builder
            .build_int_compare(inkwell::IntPredicate::UGE, shifted, tag_min, "tagged")
            .unwrap();
        self.builder.build_conditional_branch(is_tagged, check_tag, print_num).unwrap();

        // ── print_num ──────────────────────────────────────────────────────
        self.builder.position_at_end(print_num);
        let fmt_g = self.make_global_str("%g");
        let as_f64 = self.builder.build_bit_cast(val, self.f64_ty, "f").unwrap();
        self.builder.build_call(printf_fn, &[fmt_g.into(), as_f64.into()], "").unwrap();
        self.builder.build_unconditional_branch(done).unwrap();

        // ── check_tag: switch on upper 16 bits ─────────────────────────────
        self.builder.position_at_end(check_tag);
        self.builder.build_switch(
            shifted,
            print_undef,
            &[
                (self.i64_ty.const_int(0xFFF1, false), print_undef),
                (self.i64_ty.const_int(0xFFF2, false), print_null),
                (self.i64_ty.const_int(0xFFF3, false), print_bool_f),
                (self.i64_ty.const_int(0xFFF4, false), print_bool_t),
                (self.i64_ty.const_int(0xFFF6, false), print_obj),
                (self.i64_ty.const_int(0xFFF7, false), print_str),
                (self.i64_ty.const_int(0xFFF8, false), print_symbol),
                (self.i64_ty.const_int(0xFFF9, false), print_closure),
            ],
        ).unwrap();

        // ── print true ─────────────────────────────────────────────────────
        self.builder.position_at_end(print_bool_t);
        let s_true = self.make_global_str("true");
        let fmt_s = self.make_global_str("%s");
        self.builder.build_call(printf_fn, &[fmt_s.into(), s_true.into()], "").unwrap();
        self.builder.build_unconditional_branch(done).unwrap();

        // ── print false ────────────────────────────────────────────────────
        self.builder.position_at_end(print_bool_f);
        let s_false = self.make_global_str("false");
        let fmt_s2 = self.make_global_str("%s");
        self.builder.build_call(printf_fn, &[fmt_s2.into(), s_false.into()], "").unwrap();
        self.builder.build_unconditional_branch(done).unwrap();

        // ── print string ───────────────────────────────────────────────────
        self.builder.position_at_end(print_str);
        let payload = self.builder
            .build_and(val, self.i64_ty.const_int(crate::nan_box::PAYLOAD_MASK, false), "payload")
            .unwrap();
        let sptr = self.builder
            .build_int_to_ptr(payload, self.ptr_ty, "sptr")
            .unwrap();
        let fmt_s3 = self.make_global_str("%s");
        self.builder.build_call(printf_fn, &[fmt_s3.into(), sptr.into()], "").unwrap();
        self.builder.build_unconditional_branch(done).unwrap();

        // ── print object ───────────────────────────────────────────────────
        self.builder.position_at_end(print_obj);
        let payload_obj = self.builder
            .build_and(val, self.i64_ty.const_int(crate::nan_box::PAYLOAD_MASK, false), "payload")
            .unwrap();
        let obj_ptr = self.builder
            .build_int_to_ptr(payload_obj, self.ptr_ty, "obj_ptr")
            .unwrap();
        let vtable_ptr = self.builder.build_load(self.ptr_ty, obj_ptr, "vtable_ptr").unwrap().into_pointer_value();
        
        let vtable_addr = self.builder.build_ptr_to_int(vtable_ptr, self.i64_ty, "vtable_addr").unwrap();
        let has_vtable = self.builder.build_int_compare(inkwell::IntPredicate::NE, vtable_addr, self.i64_ty.const_int(0, false), "has_vtable").unwrap();
        
        let load_name_bb = self.ctx.append_basic_block(func, "load_name");
        let default_name_bb = self.ctx.append_basic_block(func, "default_name");

        self.builder.build_conditional_branch(has_vtable, load_name_bb, default_name_bb).unwrap();

        // ── load_name ──
        self.builder.position_at_end(load_name_bb);
        let name_offset = self.i32_ty.const_int(1, false);
        let name_ptr_ptr = unsafe {
            self.builder.build_gep(self.ptr_ty, vtable_ptr, &[name_offset], "name_ptr_ptr").unwrap()
        };
        let loaded_name = self.builder.build_load(self.ptr_ty, name_ptr_ptr, "name_ptr").unwrap().into_pointer_value();
        let fmt_obj = self.make_global_str("%s {}");
        self.builder.build_call(printf_fn, &[fmt_obj.into(), loaded_name.into()], "").unwrap();
        self.builder.build_unconditional_branch(done).unwrap();

        // ── default_name ──
        self.builder.position_at_end(default_name_bb);
        let default_name_ptr = self.make_global_str("Object");
        let fmt_obj_default = self.make_global_str("%s {}");
        self.builder.build_call(printf_fn, &[fmt_obj_default.into(), default_name_ptr.into()], "").unwrap();
        self.builder.build_unconditional_branch(done).unwrap();

        // ── print null ─────────────────────────────────────────────────────
        self.builder.position_at_end(print_null);
        let s_null = self.make_global_str("null");
        let fmt_s4 = self.make_global_str("%s");
        self.builder.build_call(printf_fn, &[fmt_s4.into(), s_null.into()], "").unwrap();
        self.builder.build_unconditional_branch(done).unwrap();

        // ── print undefined ────────────────────────────────────────────────
        self.builder.position_at_end(print_undef);
        let s_undef = self.make_global_str("undefined");
        let fmt_s5 = self.make_global_str("%s");
        self.builder.build_call(printf_fn, &[fmt_s5.into(), s_undef.into()], "").unwrap();
        self.builder.build_unconditional_branch(done).unwrap();

        // ── print closure ──────────────────────────────────────────────────
        self.builder.position_at_end(print_closure);
        let s_closure = self.make_global_str("[Function]");
        let fmt_s6 = self.make_global_str("%s");
        self.builder.build_call(printf_fn, &[fmt_s6.into(), s_closure.into()], "").unwrap();
        self.builder.build_unconditional_branch(done).unwrap();

        // ── print symbol ───────────────────────────────────────────────────
        self.builder.position_at_end(print_symbol);
        // Call __bs_String(val) to get "Symbol(desc)" string
        let bs_string_fn = self.funcs["__bs_String"];
        let sym_str = self.builder.build_call(bs_string_fn, &[val.into()], "sym_str").unwrap()
            .try_as_basic_value().basic().unwrap().into_int_value();
        let sym_payload = self.builder
            .build_and(sym_str, self.i64_ty.const_int(crate::nan_box::PAYLOAD_MASK, false), "sym_payload")
            .unwrap();
        let sym_sptr = self.builder
            .build_int_to_ptr(sym_payload, self.ptr_ty, "sym_sptr")
            .unwrap();
        let fmt_s_sym = self.make_global_str("%s");
        self.builder.build_call(printf_fn, &[fmt_s_sym.into(), sym_sptr.into()], "").unwrap();
        self.builder.build_unconditional_branch(done).unwrap();

        // ── done: newline + return ─────────────────────────────────────────
        self.builder.position_at_end(done);
        self.builder
            .build_call(putchar_fn, &[self.i32_ty.const_int(10, false).into()], "")
            .unwrap();
        self.builder.build_return(None).unwrap();
    }

    // ── helpers ────────────────────────────────────────────────────────────

    pub fn flush_deferred_clears(&mut self) {
        let clears: Vec<_> = self.deferred_clears.drain(..).collect();
        for reg in clears {
            let undef_val = self.i64_ty.const_int(crate::nan_box::TAG_UNDEFINED, false);
            self.store(reg, undef_val);
        }
    }

    /// Resolve a `MirOperand` to an LLVM `i64` value.
    pub fn val(&mut self, op: &MirOperand) -> CompileResult<IntValue<'ctx>> {
        match op {
            MirOperand::Reg(r) => {
                let a = self.regs.get(r).ok_or_else(|| CompileError::Codegen {
                    message: format!("unknown reg r{}", r),
                })?;
                Ok(self.builder.build_load(self.i64_ty, *a, &format!("r{}", r)).unwrap().into_int_value())
            }
            MirOperand::ConstNum(n) => Ok(self.nan.const_number(*n)),
            MirOperand::ConstBool(b) => Ok(self.nan.const_bool(*b)),
            MirOperand::ConstStr(s) => {
                let gp = self.make_global_str(s);
                Ok(self.nan.box_string_ptr(&self.builder, gp))
            }
            MirOperand::ConstNull => Ok(self.nan.const_null()),
            MirOperand::ConstUndefined => Ok(self.nan.const_undefined()),
        }
    }

    pub(super) fn store(&self, reg: MirReg, v: IntValue<'ctx>) {
        if let Some(&a) = self.regs.get(&reg) {
            self.builder.build_store(a, v).unwrap();
        }
    }

    pub(super) fn make_global_str(&mut self, s: &str) -> PointerValue<'ctx> {
        if let Some(g) = self.str_cache.get(s) {
            return g.as_pointer_value();
        }
        let bytes = s.as_bytes();
        let arr_ty = self.i8_ty.array_type(bytes.len() as u32 + 1);
        let name = format!(".str.{}", self.str_counter);
        self.str_counter += 1;
        let g = self.module.add_global(arr_ty, Some(AddressSpace::default()), &name);
        let vals: Vec<IntValue<'ctx>> = bytes
            .iter()
            .chain(std::iter::once(&0u8))
            .map(|&b| self.i8_ty.const_int(b as u64, false))
            .collect();
        g.set_initializer(&self.i8_ty.const_array(&vals));
        g.set_constant(true);
        g.set_unnamed_addr(true);
        self.str_cache.insert(s.to_string(), g);
        g.as_pointer_value()
    }
}
