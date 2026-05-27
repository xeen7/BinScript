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
use inkwell::{FloatPredicate, IntPredicate};

use diagnostics::{CompileError, CompileResult};
use mir::types::*;
use mir::BuiltinFn;

use crate::nan_box::NanBoxHelper;

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
    vtables: HashMap<String, GlobalValue<'ctx>>,
    classes: HashMap<String, hir::HirClass>,

    // Generator Codegen State
    gen_state_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    gen_sent_val: Option<inkwell::values::IntValue<'ctx>>,
    gen_state_ty: Option<inkwell::types::StructType<'ctx>>,
    resume_blocks: HashMap<u32, inkwell::basic_block::BasicBlock<'ctx>>,
    gen_num_args: u32,
    shadow_frame_ty: inkwell::types::StructType<'ctx>,
}

impl<'ctx> LlvmCodegen<'ctx> {
    pub fn new(ctx: &'ctx Context, module_name: &str) -> Self {
        let module = ctx.create_module(module_name);
        let builder = ctx.create_builder();
        let i64_ty = ctx.i64_type();
        let i32_ty = ctx.i32_type();
        let i8_ty = ctx.i8_type();
        let f64_ty = ctx.f64_type();
        let ptr_ty = ctx.ptr_type(AddressSpace::default());
        let void_ty = ctx.void_type();
        let nan = NanBoxHelper::new(ctx, i64_ty, f64_ty, ctx.bool_type());
        
        let shadow_frame_ty = ctx.struct_type(&[
            ptr_ty.into(), // prev
            i32_ty.into(), // num_roots
            i32_ty.into(), // _pad
            ptr_ty.into(), // roots
        ], false);

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
            vtables: HashMap::new(),
            classes: HashMap::new(),
            gen_state_ptr: None,
            gen_sent_val: None,
            gen_state_ty: None,
            resume_blocks: HashMap::new(),
            gen_num_args: 0,
            shadow_frame_ty,
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

        // Generate static global vtables (references user functions).
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

        // ptr __bs_alloc(ptr vtable_ptr, i64 size_in_bytes) -> i64
        add(self, "__bs_alloc", self.i64_ty.fn_type(&[self.ptr_ty.into(), self.i64_ty.into()], false));
        // i64 __bs_instanceof(i64 obj, i64 shape_id) -> i64
        add(self, "__bs_instanceof", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        // ptr __bs_alloc_closure(i64 size_in_bytes) -> i64
        add(self, "__bs_alloc_closure", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        // ptr __bs_alloc_generator(i64 size_in_bytes) -> i64
        add(self, "__bs_alloc_generator", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        // i64 __bs_generator_next(i64 gen_ptr, i64 sent_val) -> i64
        add(self, "__bs_generator_next", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        // i64 __bs_generator_is_done(i64 gen_ptr) -> i64
        add(self, "__bs_generator_is_done", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        // void __bs_drain_microtasks()
        add(self, "__bs_drain_microtasks", self.void_ty.fn_type(&[], false));
        // i64 __bs_promise_new()
        add(self, "__bs_promise_new", self.i64_ty.fn_type(&[], false));
        // void __bs_promise_resolve(i64 promise_tagged, i64 value_tagged)
        add(self, "__bs_promise_resolve", self.void_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        // i64 __bs_promise_then(i64 promise_tagged, i64 closure_tagged) -> i64 promise
        add(self, "__bs_promise_then", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        // i64 __bs_async_drive(i64 generator_tagged) -> i64 promise
        add(self, "__bs_async_drive", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        
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
        add(self, "__bs_parseInt_1", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        add(self, "__bs_parseInt_2", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        add(self, "__bs_parseFloat", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        add(self, "__bs_isNaN", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        add(self, "__bs_isFinite", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        add(self, "__bs_number_isInteger", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        add(self, "__bs_typeof", self.i64_ty.fn_type(&[self.i64_ty.into()], false));

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

        add(self, "__bs_call_push", dispatch_1);
        add(self, "__bs_call_pop", dispatch_0);
        add(self, "__bs_call_slice", dispatch_2);
        add(self, "__bs_call_indexOf", dispatch_1);
        add(self, "__bs_call_includes", dispatch_1);
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
        add(self, "__bs_call_toString", dispatch_0);
        add(self, "__bs_call_valueOf", dispatch_0);

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

        // --- GC & Shadow Stack ---
        // void __bs_shadow_push(ptr frame)
        add(self, "__bs_shadow_push", self.void_ty.fn_type(&[self.ptr_ty.into()], false));
        // void __bs_shadow_pop()
        add(self, "__bs_shadow_pop", self.void_ty.fn_type(&[], false));
        // void __bs_safepoint_poll()
        add(self, "__bs_safepoint_poll", self.void_ty.fn_type(&[], false));

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
            .build_int_compare(IntPredicate::UGE, shifted, tag_min, "tagged")
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

    // ── static global vtables ──────────────────────────────────────────────

    fn emit_vtables(&mut self, mir: &MirModule) -> CompileResult<()> {
        let mut method_names: Vec<String> = mir.classes.values()
            .flat_map(|c| c.methods.iter().map(|m| m.name.clone()))
            .collect();
        method_names.sort();
        method_names.dedup();

        let mut class_names: Vec<String> = mir.classes.keys().cloned().collect();
        class_names.sort();
        let class_shapes: HashMap<String, u64> = class_names
            .iter()
            .enumerate()
            .map(|(i, name)| (name.clone(), (i + 1) as u64))
            .collect();

        // Structural type: { parent: ptr, name: ptr, shape_id: i64, fields_count: i64, field_names: ptr, slots: [num_methods x ptr] }
        let mut vtable_fields = vec![
            self.ptr_ty.into(), // parent
            self.ptr_ty.into(), // name
            self.i64_ty.into(), // shape_id
            self.i64_ty.into(), // fields_count
            self.ptr_ty.into(), // field_names
        ];
        for _ in 0..method_names.len() {
            vtable_fields.push(self.ptr_ty.into());
        }
        let vtable_ty = self.ctx.struct_type(&vtable_fields, false);

        // First, declare all vtable globals.
        for class_name in &class_names {
            let g = self.module.add_global(vtable_ty, None, &format!("__bs_class_{}_vtable", class_name));
            self.vtables.insert(class_name.clone(), g);
        }

        // Initialize each vtable.
        for class_name in &class_names {
            let class = &mir.classes[class_name];
            let shape_id = class_shapes[class_name];
            let g = self.vtables[class_name];

            let parent_val = if let Some(ref super_name) = class.super_name {
                let super_g = self.vtables[super_name];
                super_g.as_pointer_value()
            } else {
                self.ptr_ty.const_null()
            };

            let name_val = self.make_global_str(class_name);
            let shape_val = self.i64_ty.const_int(shape_id, false);

            // Build the static global array of field names for this class
            let fields = &class.fields;
            let fields_count_val = self.i64_ty.const_int(fields.len() as u64, false);
            let field_names_array_val = if fields.is_empty() {
                self.ptr_ty.const_null()
            } else {
                let mut field_ptrs = Vec::new();
                for f in fields {
                    field_ptrs.push(self.make_global_str(f).into());
                }
                let array_ty = self.ptr_ty.array_type(fields.len() as u32);
                let array_const = self.ptr_ty.const_array(&field_ptrs);
                let array_global = self.module.add_global(array_ty, None, &format!("__bs_class_{}_field_names", class_name));
                array_global.set_initializer(&array_const);
                array_global.set_constant(true);
                array_global.as_pointer_value()
            };

            let mut vals = vec![
                parent_val.into(),
                name_val.into(),
                shape_val.into(),
                fields_count_val.into(),
                field_names_array_val.into(),
            ];

            for m_name in &method_names {
                let slot_val = if let Some(impl_class) = self.find_method_impl(mir, class_name, m_name) {
                    let fn_name = format!("__bs_class_{}_{}", impl_class, m_name);
                    let fv = self.funcs.get(&fn_name).cloned().ok_or_else(|| {
                        CompileError::Codegen {
                            message: format!("Method implementation function {} not found", fn_name),
                        }
                    })?;
                    fv.as_global_value().as_pointer_value()
                } else {
                    self.ptr_ty.const_null()
                };
                vals.push(slot_val.into());
            }

            let init = self.ctx.const_struct(&vals, false);
            g.set_initializer(&init);
            g.set_constant(true);
        }

        Ok(())
    }

    fn find_method_impl(&self, mir: &MirModule, class_name: &str, method_name: &str) -> Option<String> {
        let mut curr = class_name;
        while let Some(class) = mir.classes.get(curr) {
            if class.methods.iter().any(|m| m.name == method_name) {
                return Some(curr.to_string());
            }
            if let Some(ref super_name) = class.super_name {
                curr = super_name;
            } else {
                break;
            }
        }
        None
    }

    fn get_all_fields_count(&self, class_name: &str) -> usize {
        let mut count = 0;
        if let Some(class) = self.classes.get(class_name) {
            if let Some(ref super_name) = class.super_name {
                count += self.get_all_fields_count(super_name);
            }
            count += class.fields.len();
        }
        count
    }

    // ── user function emission ─────────────────────────────────────────────

    fn emit_function(&mut self, func: &MirFunction) -> CompileResult<()> {
        if func.is_generator || func.is_async {
            self.emit_generator_function(func)
        } else {
            self.emit_normal_function(func)
        }
    }

    fn emit_normal_function(&mut self, func: &MirFunction) -> CompileResult<()> {
        let fv = self.funcs[&func.name];
        self.regs.clear();
        self.bbs.clear();

        // Create LLVM basic blocks.
        for b in &func.blocks {
            let bb = self.ctx.append_basic_block(fv, &format!("bb{}", b.id));
            self.bbs.insert(b.id, bb);
        }

        // Allocas in the entry block.
        let entry = self.bbs[&func.blocks[0].id];
        self.builder.position_at_end(entry);
        
        let regs_array = self.builder.build_alloca(self.i64_ty.array_type(func.next_reg as u32), "regs_array").unwrap();
        let regs_array_ptr = self.builder.build_int_to_ptr(self.builder.build_ptr_to_int(regs_array, self.i64_ty, "").unwrap(), self.ptr_ty, "regs_ptr").unwrap();
        
        for rid in 0..func.next_reg {
            let a = unsafe { self.builder.build_gep(self.i64_ty, regs_array, &[self.i32_ty.const_int(rid as u64, false)], &format!("r{}", rid)).unwrap() };
            self.builder.build_store(a, self.nan.const_undefined()).unwrap();
            self.regs.insert(rid, a);
        }

        // Shadow stack push
        let shadow_frame = self.builder.build_alloca(self.shadow_frame_ty, "shadow_frame").unwrap();
        let num_roots_ptr = self.builder.build_struct_gep(self.shadow_frame_ty, shadow_frame, 1, "num_roots_ptr").unwrap();
        self.builder.build_store(num_roots_ptr, self.i32_ty.const_int(func.next_reg as u64, false)).unwrap();
        let roots_ptr = self.builder.build_struct_gep(self.shadow_frame_ty, shadow_frame, 3, "roots_ptr").unwrap();
        self.builder.build_store(roots_ptr, regs_array_ptr).unwrap();
        let shadow_push_fn = self.module.get_function("__bs_shadow_push").unwrap();
        self.builder.build_call(shadow_push_fn, &[shadow_frame.into()], "shadow_push").unwrap();

        // Store incoming parameters.
        for (i, (reg, _)) in func.params.iter().enumerate() {
            let pv = fv.get_nth_param(i as u32)
                .unwrap_or_else(|| panic!("Function {} has {} params in MIR but LLVM fn has fewer! Failed on param index {}", func.name, func.params.len(), i))
                .into_int_value();
            if let Some(&a) = self.regs.get(reg) {
                self.builder.build_store(a, pv).unwrap();
            }
        }

        // Emit instructions per block.
        for b in &func.blocks {
            self.builder.position_at_end(self.bbs[&b.id]);
            for instr in &b.instrs {
                self.emit_instr(instr)?;
            }
            // Ensure block has a terminator.
            if self.bbs[&b.id].get_terminator().is_none() {
                let shadow_pop_fn = self.module.get_function("__bs_shadow_pop").unwrap();
                self.builder.build_call(shadow_pop_fn, &[], "shadow_pop").unwrap();
                self.builder.build_return(Some(&self.nan.const_undefined())).unwrap();
            }
        }
        Ok(())
    }

    fn emit_generator_function(&mut self, func: &MirFunction) -> CompileResult<()> {
        let fv = self.funcs[&func.name];
        let num_args = func.params.len() as u32;
        let num_locals = func.next_reg;

        let mut struct_fields = vec![
            self.i64_ty.into(), // state_idx
            self.i64_ty.into(), // poll_fn ptr
        ];
        for _ in 0..num_args { struct_fields.push(self.i64_ty.into()); }
        for _ in 0..num_locals { struct_fields.push(self.i64_ty.into()); }

        let state_ty = self.ctx.struct_type(&struct_fields, false);
        let size_val = state_ty.size_of().unwrap();

        let poll_fn_name = format!("{}_poll", func.name);
        let poll_fn_ty = self.i64_ty.fn_type(&[self.ptr_ty.into(), self.i64_ty.into()], false);
        let poll_fv = self.module.add_function(&poll_fn_name, poll_fn_ty, None);

        let wrapper_bb = self.ctx.append_basic_block(fv, "entry");
        self.builder.position_at_end(wrapper_bb);

        let alloc_gen_fn = self.module.get_function("__bs_alloc_generator").unwrap();
        let alloc_call = self.builder.build_call(alloc_gen_fn, &[size_val.into()], "gen_alloc").unwrap();
        let gen_ptr_tagged = alloc_call.try_as_basic_value().basic().unwrap().into_int_value();

        let payload = self.builder.build_and(gen_ptr_tagged, self.i64_ty.const_int(crate::nan_box::PAYLOAD_MASK, false), "payload").unwrap();
        let state_ptr = self.builder.build_int_to_ptr(payload, self.ptr_ty, "state_ptr").unwrap();

        let state_idx_ptr = self.builder.build_struct_gep(state_ty, state_ptr, 0, "state_idx_ptr").unwrap();
        self.builder.build_store(state_idx_ptr, self.i64_ty.const_int(0, false)).unwrap();

        let poll_fn_ptr = poll_fv.as_global_value().as_pointer_value();
        let poll_fn_i64 = self.builder.build_ptr_to_int(poll_fn_ptr, self.i64_ty, "poll_fn_i64").unwrap();
        let poll_slot_ptr = self.builder.build_struct_gep(state_ty, state_ptr, 1, "poll_slot").unwrap();
        self.builder.build_store(poll_slot_ptr, poll_fn_i64).unwrap();

        for (i, _) in func.params.iter().enumerate() {
            let arg_val = fv.get_nth_param(i as u32).unwrap().into_int_value();
            let arg_slot = self.builder.build_struct_gep(state_ty, state_ptr, 2 + i as u32, "arg_slot").unwrap();
            self.builder.build_store(arg_slot, arg_val).unwrap();
        }

        if func.is_async {
            let drive_fn = self.module.get_function("__bs_async_drive").unwrap();
            let drive_call = self.builder.build_call(drive_fn, &[gen_ptr_tagged.into()], "drive_async").unwrap();
            let promise_ptr = drive_call.try_as_basic_value().basic().unwrap().into_int_value();
            self.builder.build_return(Some(&promise_ptr)).unwrap();
        } else {
            self.builder.build_return(Some(&gen_ptr_tagged)).unwrap();
        }
        self.regs.clear();
        self.bbs.clear();
        self.resume_blocks.clear();
        self.gen_state_ty = Some(state_ty);
        self.gen_num_args = num_args;

        let poll_entry = self.ctx.append_basic_block(poll_fv, "entry");
        self.builder.position_at_end(poll_entry);

        let state_arg = poll_fv.get_nth_param(0).unwrap().into_pointer_value();
        let sent_val_arg = poll_fv.get_nth_param(1).unwrap().into_int_value();
        self.gen_state_ptr = Some(state_arg);
        self.gen_sent_val = Some(sent_val_arg);

        let regs_array = self.builder.build_alloca(self.i64_ty.array_type(func.next_reg as u32), "regs_array").unwrap();
        let regs_array_ptr = self.builder.build_int_to_ptr(self.builder.build_ptr_to_int(regs_array, self.i64_ty, "").unwrap(), self.ptr_ty, "regs_ptr").unwrap();

        for (i, (reg, _)) in func.params.iter().enumerate() {
            let arg_slot = self.builder.build_struct_gep(state_ty, state_arg, 2 + i as u32, "arg_slot").unwrap();
            let loaded = self.builder.build_load(self.i64_ty, arg_slot, "loaded_arg").unwrap().into_int_value();
            let a = unsafe { self.builder.build_gep(self.i64_ty, regs_array, &[self.i32_ty.const_int(*reg as u64, false)], &format!("r{}", reg)).unwrap() };
            self.builder.build_store(a, loaded).unwrap();
            self.regs.insert(*reg, a);
        }

        for rid in 0..func.next_reg {
            if !self.regs.contains_key(&rid) {
                let a = unsafe { self.builder.build_gep(self.i64_ty, regs_array, &[self.i32_ty.const_int(rid as u64, false)], &format!("r{}", rid)).unwrap() };
                self.builder.build_store(a, self.nan.const_undefined()).unwrap();
                self.regs.insert(rid, a);
            }
        }

        // Shadow stack push
        let shadow_frame = self.builder.build_alloca(self.shadow_frame_ty, "shadow_frame").unwrap();
        let num_roots_ptr = self.builder.build_struct_gep(self.shadow_frame_ty, shadow_frame, 1, "num_roots_ptr").unwrap();
        self.builder.build_store(num_roots_ptr, self.i32_ty.const_int(func.next_reg as u64, false)).unwrap();
        let roots_ptr = self.builder.build_struct_gep(self.shadow_frame_ty, shadow_frame, 3, "roots_ptr").unwrap();
        self.builder.build_store(roots_ptr, regs_array_ptr).unwrap();
        let shadow_push_fn = self.module.get_function("__bs_shadow_push").unwrap();
        self.builder.build_call(shadow_push_fn, &[shadow_frame.into()], "shadow_push").unwrap();

        for b in &func.blocks {
            let bb = self.ctx.append_basic_block(poll_fv, &format!("bb{}", b.id));
            self.bbs.insert(b.id, bb);
        }

        let done_bb = self.ctx.append_basic_block(poll_fv, "done");

        for i in 0..func.num_yield_points {
            let rbb = self.ctx.append_basic_block(poll_fv, &format!("resume_{}", i));
            self.resume_blocks.insert(i, rbb);
        }

        let state_idx_ptr = self.builder.build_struct_gep(state_ty, state_arg, 0, "state_idx_ptr").unwrap();
        let state_val = self.builder.build_load(self.i64_ty, state_idx_ptr, "state_val").unwrap().into_int_value();

        let mut switch_cases = Vec::new();
        switch_cases.push((self.i64_ty.const_int(0, false), self.bbs[&func.blocks[0].id]));
        for i in 0..func.num_yield_points {
            switch_cases.push((self.i64_ty.const_int((i + 1) as u64, false), self.resume_blocks[&i]));
        }

        self.builder.build_switch(state_val, done_bb, &switch_cases).unwrap();

        self.builder.position_at_end(done_bb);
        let shadow_pop_fn = self.module.get_function("__bs_shadow_pop").unwrap();
        self.builder.build_call(shadow_pop_fn, &[], "shadow_pop").unwrap();
        self.builder.build_return(Some(&self.nan.const_undefined())).unwrap();

        for b in &func.blocks {
            self.builder.position_at_end(self.bbs[&b.id]);
            for instr in &b.instrs {
                self.emit_instr(instr)?;
            }
            if self.bbs[&b.id].get_terminator().is_none() {
                if let Some(state_ptr) = self.gen_state_ptr {
                    let state_idx_ptr = unsafe { self.builder.build_struct_gep(state_ty, state_ptr, 0, "state_idx_ptr").unwrap() };
                    self.builder.build_store(state_idx_ptr, self.i64_ty.const_all_ones()).unwrap();
                }
                let shadow_pop_fn = self.module.get_function("__bs_shadow_pop").unwrap();
                self.builder.build_call(shadow_pop_fn, &[], "shadow_pop").unwrap();
                self.builder.build_return(Some(&self.nan.const_undefined())).unwrap();
            }
        }

        self.gen_state_ptr = None;
        self.gen_sent_val = None;
        self.gen_state_ty = None;
        self.resume_blocks.clear();
        self.gen_num_args = 0;

        Ok(())
    }

    // ── main wrapper ───────────────────────────────────────────────────────

    fn emit_main(&mut self, body: &MirFunction) -> CompileResult<()> {
        let ft = self.i32_ty.fn_type(&[], false);
        let main_fn = self.module.add_function("main", ft, None);

        self.regs.clear();
        self.bbs.clear();

        for b in &body.blocks {
            let bb = self.ctx.append_basic_block(main_fn, &format!("bb{}", b.id));
            self.bbs.insert(b.id, bb);
        }

        let entry = self.bbs[&body.blocks[0].id];
        self.builder.position_at_end(entry);
        for rid in 0..body.next_reg {
            let a = self.builder.build_alloca(self.i64_ty, &format!("r{}", rid)).unwrap();
            self.builder.build_store(a, self.nan.const_undefined()).unwrap();
            self.regs.insert(rid, a);
        }

        for b in &body.blocks {
            self.builder.position_at_end(self.bbs[&b.id]);
            for instr in &b.instrs {
                // Skip Return instructions — main always returns i32 0.
                if matches!(instr, MirInstr::Return(_)) {
                    continue;
                }
                self.emit_instr(instr)?;
            }
            if self.bbs[&b.id].get_terminator().is_none() {
                // Unterminated blocks in main are either the end of the program
                // or unreachable dead blocks. MIR control flow always emits
                // explicit Jump/Branch instructions for connected blocks, so we
                // never need to fall through to the next sequential block here.
                let drain_fn = self.module.get_function("__bs_drain_microtasks").unwrap();
                self.builder.build_call(drain_fn, &[], "drain").unwrap();
                self.builder
                    .build_return(Some(&self.i32_ty.const_int(0, false)))
                    .unwrap();
            }
        }
        Ok(())
    }

    // ── instruction emission ───────────────────────────────────────────────

    fn emit_instr(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::Move(dest, src) => {
                let v = self.val(src)?;
                self.store(*dest, v);
            }

            MirInstr::Add(d, l, r) => self.emit_add(*d, l, r)?,
            MirInstr::Sub(d, l, r) => self.emit_arith_f64(*d, l, r, "fsub")?,
            MirInstr::Mul(d, l, r) => self.emit_arith_f64(*d, l, r, "fmul")?,
            MirInstr::Div(d, l, r) => self.emit_arith_f64(*d, l, r, "fdiv")?,
            MirInstr::Mod(d, l, r) => self.emit_arith_f64(*d, l, r, "frem")?,
            MirInstr::Exp(d, l, r) => {
                let lv = self.val(l)?;
                let rv = self.val(r)?;
                let func = self.module.get_function("__bs_exp").unwrap();
                let call = self.builder.build_call(func, &[lv.into(), rv.into()], "exp_call").unwrap();
                let res = call.try_as_basic_value().basic().unwrap().into_int_value();
                self.store(*d, res);
            }

            MirInstr::Plus(d, v) => {
                let vv = self.val(v)?;
                let func = self.module.get_function("__bs_Number").unwrap();
                let call = self.builder.build_call(func, &[vv.into()], "plus_call").unwrap();
                let res = call.try_as_basic_value().basic().unwrap().into_int_value();
                self.store(*d, res);
            }

            MirInstr::Neg(d, v) => {
                let vv = self.val(v)?;
                let fv = self.nan.unbox_number(&self.builder, vv);
                let r = self.builder.build_float_neg(fv, "neg").unwrap();
                self.store(*d, self.nan.box_number(&self.builder, r));
            }

            MirInstr::Lt(d, l, r) => self.emit_cmp_f64(*d, l, r, FloatPredicate::OLT)?,
            MirInstr::Le(d, l, r) => self.emit_cmp_f64(*d, l, r, FloatPredicate::OLE)?,
            MirInstr::Gt(d, l, r) => self.emit_cmp_f64(*d, l, r, FloatPredicate::OGT)?,
            MirInstr::Ge(d, l, r) => self.emit_cmp_f64(*d, l, r, FloatPredicate::OGE)?,
            MirInstr::Eq(d, l, r) | MirInstr::StrictEq(d, l, r) => {
                self.emit_eq(*d, l, r)?
            }
            MirInstr::Ne(d, l, r) | MirInstr::StrictNe(d, l, r) => {
                self.emit_ne(*d, l, r)?
            }
            MirInstr::In(d, l, r) => {
                let lv = self.val(l)?;
                let rv = self.val(r)?;
                let func = self.module.get_function("__bs_in").unwrap();
                let call = self.builder.build_call(func, &[lv.into(), rv.into()], "in_call").unwrap();
                let res = call.try_as_basic_value().basic().unwrap().into_int_value();
                self.store(*d, res);
            }

            MirInstr::Not(d, v) => {
                let vv = self.val(v)?;
                let truthy = self.nan.is_truthy(&self.builder, vv);
                let neg = self.builder.build_not(truthy, "not").unwrap();
                self.store(*d, self.nan.box_bool(&self.builder, neg));
            }

            MirInstr::CallDirect(d, name, args) => {
                let fn_val = self.funcs.get(name).copied().ok_or_else(|| {
                    CompileError::Codegen { message: format!("unknown fn {}", name) }
                })?;
                let mut av: Vec<BasicMetadataValueEnum<'ctx>> = args
                    .iter()
                    .map(|a| self.val(a).map(|v| v.into()))
                    .collect::<CompileResult<_>>()?;
                let expected_params = fn_val.count_params() as usize;
                if expected_params == av.len() + 1 {
                    av.insert(0, self.nan.const_undefined().into());
                }
                let rv = self.builder.build_call(fn_val, &av, "call").unwrap();
                let v = rv
                    .try_as_basic_value()
                    .basic()
                    .map(|bv| bv.into_int_value())
                    .unwrap_or_else(|| self.nan.const_undefined());
                self.store(*d, v);
            }

            MirInstr::CallBuiltin(d, BuiltinFn::ConsoleLog, args) => {
                let log1 = self.funcs["__bs_console_log_1"];
                for a in args {
                    let v = self.val(a)?;
                    self.builder.build_call(log1, &[v.into()], "").unwrap();
                }
                self.store(*d, self.nan.const_undefined());
            }
            MirInstr::CallBuiltin(d, BuiltinFn::GeneratorNext, args) => {
                let gen_next = self.module.get_function("__bs_generator_next").unwrap();
                let gen_ptr = self.val(&args[0])?;
                let sent_val = self.val(&args[1])?;
                let rv = self.builder.build_call(gen_next, &[gen_ptr.into(), sent_val.into()], "gen_next").unwrap();
                let v = rv.try_as_basic_value().basic().unwrap().into_int_value();
                self.store(*d, v);
            }
            MirInstr::CallBuiltin(d, BuiltinFn::GeneratorIsDone, args) => {
                let gen_is_done = self.module.get_function("__bs_generator_is_done").unwrap();
                let gen_ptr = self.val(&args[0])?;
                let rv = self.builder.build_call(gen_is_done, &[gen_ptr.into()], "gen_is_done").unwrap();
                let v = rv.try_as_basic_value().basic().unwrap().into_int_value();
                self.store(*d, v);
            }
            MirInstr::CallBuiltin(d, BuiltinFn::PromiseAll2, args) => {
                let f = self.module.get_function("__bs_promise_all_2").unwrap();
                let a1 = self.val(&args[0])?;
                let a2 = self.val(&args[1])?;
                let rv = self.builder.build_call(f, &[a1.into(), a2.into()], "promise_all_2").unwrap();
                let v = rv.try_as_basic_value().basic().unwrap().into_int_value();
                self.store(*d, v);
            }
            MirInstr::CallBuiltin(d, BuiltinFn::PromiseRace2, args) => {
                let f = self.module.get_function("__bs_promise_race_2").unwrap();
                let a1 = self.val(&args[0])?;
                let a2 = self.val(&args[1])?;
                let rv = self.builder.build_call(f, &[a1.into(), a2.into()], "promise_race_2").unwrap();
                let v = rv.try_as_basic_value().basic().unwrap().into_int_value();
                self.store(*d, v);
            }
            MirInstr::CallBuiltin(d, BuiltinFn::JsonParseLazy, args) => {
                let json_parse = self.module.get_function("__bs_json_parse_lazy").unwrap();
                if let MirOperand::ConstStr(s) = &args[0] {
                    let global_str = self.builder.build_global_string_ptr(s, "json_str").unwrap();
                    let ptr_val = global_str.as_pointer_value();
                    let len_val = self.i32_ty.const_int(s.len() as u64, false);
                    let rv = self.builder.build_call(json_parse, &[ptr_val.into(), len_val.into()], "json_parse_lazy").unwrap();
                    let v = rv.try_as_basic_value().basic().unwrap().into_int_value();
                    self.store(*d, v);
                } else {
                    return Err(CompileError::Codegen { message: "JsonParseLazy arg must be ConstStr".into() });
                }
            }
            MirInstr::CallBuiltin(_, _, _) => {
                return Err(CompileError::Codegen { message: "Unsupported BuiltinFn variant in CallBuiltin".into() });
            }

            MirInstr::Branch(cond, t, f) => {
                let cv_val = self.val(cond)?;
                let cv = self.nan.is_truthy(&self.builder, cv_val);
                let tbb = self.bbs[t];
                let fbb = self.bbs[f];
                self.builder.build_conditional_branch(cv, tbb, fbb).unwrap();
            }
            MirInstr::Jump(target) => {
                let current_block = self.builder.get_insert_block().unwrap();
                let current_block_name_cstr = current_block.get_name();
                let current_block_name = current_block_name_cstr.to_str().unwrap();
                let bb = self.bbs[target];
                
                // If target block ID is less than current block ID (or we can't easily tell so we just check the target name vs current),
                // it's a back-edge. A simple heuristic for now is to just look if the target id is <= current block's parsed id.
                let mut is_back_edge = false;
                if let Some(curr_id_str) = current_block_name.strip_prefix("bb") {
                    if let Ok(curr_id) = curr_id_str.parse::<u32>() {
                        if *target <= curr_id {
                            is_back_edge = true;
                        }
                    }
                }
                
                if is_back_edge {
                    let safepoint_poll = self.module.get_function("__bs_safepoint_poll").unwrap();
                    self.builder.build_call(safepoint_poll, &[], "safepoint_poll").unwrap();
                }

                self.builder.build_unconditional_branch(bb).unwrap();
            }
            MirInstr::Suspend(idx, val) => {
                let v = self.val(val)?;
                let state_ptr = self.gen_state_ptr.unwrap();
                let state_ty = self.gen_state_ty.unwrap();
                
                for (rid, alloca) in &self.regs {
                    let val_to_save = self.builder.build_load(self.i64_ty, *alloca, "saved").unwrap().into_int_value();
                    let slot = unsafe { self.builder.build_struct_gep(state_ty, state_ptr, 2 + self.gen_num_args + *rid, "slot").unwrap() };
                    self.builder.build_store(slot, val_to_save).unwrap();
                }
                
                let state_idx_ptr = unsafe { self.builder.build_struct_gep(state_ty, state_ptr, 0, "state_idx_ptr").unwrap() };
                self.builder.build_store(state_idx_ptr, self.i64_ty.const_int((*idx + 1) as u64, false)).unwrap();
                
                self.builder.build_return(Some(&v)).unwrap();
            }
            MirInstr::Resume(dest, idx) => {
                let resume_bb = self.resume_blocks[idx];
                self.builder.position_at_end(resume_bb);
                
                let state_ptr = self.gen_state_ptr.unwrap();
                let state_ty = self.gen_state_ty.unwrap();
                
                for (rid, alloca) in &self.regs {
                    let slot = unsafe { self.builder.build_struct_gep(state_ty, state_ptr, 2 + self.gen_num_args + *rid, "slot").unwrap() };
                    let loaded = self.builder.build_load(self.i64_ty, slot, "restored").unwrap().into_int_value();
                    self.builder.build_store(*alloca, loaded).unwrap();
                }
                
                let sent_val = self.gen_sent_val.unwrap();
                self.store(*dest, sent_val);
            }
            MirInstr::Return(v) => {
                let rv = match v {
                    Some(op) => self.val(op)?,
                    None => self.nan.const_undefined(),
                };
                if let Some(state_ptr) = self.gen_state_ptr {
                    let state_ty = self.gen_state_ty.unwrap();
                    let state_idx_ptr = unsafe { self.builder.build_struct_gep(state_ty, state_ptr, 0, "state_idx_ptr").unwrap() };
                    self.builder.build_store(state_idx_ptr, self.i64_ty.const_all_ones()).unwrap();
                }
                self.builder.build_return(Some(&rv)).unwrap();
            }

            // --- Stage 2 additions ---
            MirInstr::Alloc(dest, class_name) => {
                let fields_count = self.get_all_fields_count(class_name);
                let size_in_bytes = 8 * (1 + fields_count);
                let vtable_g = self.vtables.get(class_name).ok_or_else(|| {
                    CompileError::Codegen {
                        message: format!("Vtable not found for class {}", class_name),
                    }
                })?;
                let vtable_ptr = vtable_g.as_pointer_value();

                let alloc_fn = self.funcs["__bs_alloc"];
                let size_val = self.i64_ty.const_int(size_in_bytes as u64, false);

                let obj_val = self.builder.build_call(alloc_fn, &[vtable_ptr.into(), size_val.into()], "alloc").unwrap()
                    .try_as_basic_value()
                    .basic()
                    .unwrap()
                    .into_int_value();

                self.store(*dest, obj_val);
            }

            MirInstr::LoadField(dest, obj_reg, index) => {
                let obj_val = self.val(&MirOperand::Reg(*obj_reg))?;
                let payload = self.builder.build_and(obj_val, self.i64_ty.const_int(crate::nan_box::PAYLOAD_MASK, false), "payload").unwrap();
                let obj_ptr = self.builder.build_int_to_ptr(payload, self.ptr_ty, "obj_ptr").unwrap();

                let offset = self.i32_ty.const_int((1 + index) as u64, false);
                let field_ptr = unsafe {
                    self.builder.build_gep(self.i64_ty, obj_ptr, &[offset], "field_ptr").unwrap()
                };
                let loaded_val = self.builder.build_load(self.i64_ty, field_ptr, "loaded").unwrap().into_int_value();
                self.store(*dest, loaded_val);
            }

            MirInstr::StoreField(obj_reg, index, val_operand) => {
                let obj_val = self.val(&MirOperand::Reg(*obj_reg))?;
                let payload = self.builder.build_and(obj_val, self.i64_ty.const_int(crate::nan_box::PAYLOAD_MASK, false), "payload").unwrap();
                let obj_ptr = self.builder.build_int_to_ptr(payload, self.ptr_ty, "obj_ptr").unwrap();

                let offset = self.i32_ty.const_int((1 + index) as u64, false);
                let field_ptr = unsafe {
                    self.builder.build_gep(self.i64_ty, obj_ptr, &[offset], "field_ptr").unwrap()
                };
                let val_to_store = self.val(val_operand)?;
                self.builder.build_store(field_ptr, val_to_store).unwrap();
            }

            MirInstr::LoadProp(dest, obj_reg, prop_name) => {
                let prop_get_fn = self.module.get_function("__bs_prop_get").unwrap();
                let obj_val = self.val(&MirOperand::Reg(*obj_reg))?;
                let global_str = self.builder.build_global_string_ptr(prop_name, "prop_str").unwrap();
                let prop_ptr = global_str.as_pointer_value();
                let prop_len = self.i32_ty.const_int(prop_name.len() as u64, false);
                
                let rv = self.builder.build_call(prop_get_fn, &[obj_val.into(), prop_ptr.into(), prop_len.into()], "prop_get").unwrap();
                let v = rv.try_as_basic_value().basic().unwrap().into_int_value();
                self.store(*dest, v);
            }

            MirInstr::StoreProp(obj_reg, prop_name, val_operand) => {
                let prop_set_fn = self.module.get_function("__bs_prop_set").unwrap();
                let obj_val = self.val(&MirOperand::Reg(*obj_reg))?;
                let global_str = self.builder.build_global_string_ptr(prop_name, "prop_str").unwrap();
                let prop_ptr = global_str.as_pointer_value();
                let prop_len = self.i32_ty.const_int(prop_name.len() as u64, false);
                let val = self.val(val_operand)?;
                
                self.builder.build_call(prop_set_fn, &[obj_val.into(), prop_ptr.into(), prop_len.into(), val.into()], "prop_set").unwrap();
            }

            MirInstr::CallVTable(dest, obj_reg, method_index, args) => {
                let obj_val = self.val(&MirOperand::Reg(*obj_reg))?;
                let payload = self.builder.build_and(obj_val, self.i64_ty.const_int(crate::nan_box::PAYLOAD_MASK, false), "payload").unwrap();
                let obj_ptr = self.builder.build_int_to_ptr(payload, self.ptr_ty, "obj_ptr").unwrap();

                let vtable_ptr = self.builder.build_load(self.ptr_ty, obj_ptr, "vtable_ptr").unwrap().into_pointer_value();

                let method_offset = self.i32_ty.const_int((5 + method_index) as u64, false);
                let method_fn_ptr_ptr = unsafe {
                    self.builder.build_gep(self.ptr_ty, vtable_ptr, &[method_offset], "method_fn_ptr_ptr").unwrap()
                };
                let method_fn_ptr = self.builder.build_load(self.ptr_ty, method_fn_ptr_ptr, "method_fn_ptr").unwrap().into_pointer_value();

                let mut param_types = Vec::new();
                for _ in 0..args.len() {
                    param_types.push(self.i64_ty.into());
                }
                let fn_ty = self.i64_ty.fn_type(&param_types, false);

                let av: Vec<BasicMetadataValueEnum<'ctx>> = args
                    .iter()
                    .map(|a| self.val(a).map(|v| v.into()))
                    .collect::<CompileResult<_>>()?;
                let rv = self.builder.build_indirect_call(fn_ty, method_fn_ptr, &av, "vcall").unwrap();
                let v = rv
                    .try_as_basic_value()
                    .basic()
                    .map(|bv| bv.into_int_value())
                    .unwrap_or_else(|| self.nan.const_undefined());
                self.store(*dest, v);
            }

            MirInstr::DeleteProp(dest, obj, prop) => {
                let ov = self.val(obj)?;
                let pv = self.val(prop)?;
                let func = self.module.get_function("__bs_delete_prop").unwrap();
                let call = self.builder.build_call(func, &[ov.into(), pv.into()], "del_prop_call").unwrap();
                let res = call.try_as_basic_value().basic().unwrap().into_int_value();
                self.store(*dest, res);
            }
            MirInstr::AllocClosure(dest, func_id, captures) => {
                let func_name = self.func_id_to_name.get(func_id).ok_or_else(|| {
                    CompileError::Codegen {
                        message: format!("unknown func_id {}", func_id),
                    }
                })?;
                let fv = self.funcs.get(func_name).copied().ok_or_else(|| {
                    CompileError::Codegen {
                        message: format!("unknown fn {}", func_name),
                    }
                })?;

                // Calculate allocation size: 8 * (2 + captures.len()) bytes
                let size_in_bytes = 8 * (2 + captures.len());
                let alloc_fn = self.funcs["__bs_alloc_closure"];
                let size_val = self.i64_ty.const_int(size_in_bytes as u64, false);

                // Call __bs_alloc_closure(size)
                let closure_val = self.builder.build_call(alloc_fn, &[size_val.into()], "alloc_closure").unwrap()
                    .try_as_basic_value()
                    .basic()
                    .unwrap()
                    .into_int_value();

                // Extract raw closure pointer
                let payload = self.builder.build_and(closure_val, self.i64_ty.const_int(crate::nan_box::PAYLOAD_MASK, false), "payload").unwrap();
                let closure_ptr = self.builder.build_int_to_ptr(payload, self.ptr_ty, "closure_ptr").unwrap();

                // Store function pointer at offset 0
                let fn_ptr = fv.as_global_value().as_pointer_value();
                let offset0 = self.i32_ty.const_int(0, false);
                let fn_slot = unsafe {
                    self.builder.build_gep(self.ptr_ty, closure_ptr, &[offset0], "fn_slot").unwrap()
                };
                self.builder.build_store(fn_slot, fn_ptr).unwrap();

                // Store undefined at offset 1
                let offset1 = self.i32_ty.const_int(1, false);
                let unused_slot = unsafe {
                    self.builder.build_gep(self.i64_ty, closure_ptr, &[offset1], "unused_slot").unwrap()
                };
                self.builder.build_store(unused_slot, self.nan.const_undefined()).unwrap();

                // Store each capture at offset 2 + i
                for (i, cap) in captures.iter().enumerate() {
                    let val_to_store = self.val(cap)?;
                    let offset = self.i32_ty.const_int((2 + i) as u64, false);
                    let capture_slot = unsafe {
                        self.builder.build_gep(self.i64_ty, closure_ptr, &[offset], "capture_slot").unwrap()
                    };
                    self.builder.build_store(capture_slot, val_to_store).unwrap();
                }

                // Store tagged pointer in dest
                self.store(*dest, closure_val);
            }

            MirInstr::CallClosure(dest, callee_reg, args) => {
                let callee_val = self.val(&MirOperand::Reg(*callee_reg))?;
                let payload = self.builder.build_and(callee_val, self.i64_ty.const_int(crate::nan_box::PAYLOAD_MASK, false), "payload").unwrap();
                let closure_ptr = self.builder.build_int_to_ptr(payload, self.ptr_ty, "closure_ptr").unwrap();

                // Load function pointer from offset 0
                let offset0 = self.i32_ty.const_int(0, false);
                let fn_slot = unsafe {
                    self.builder.build_gep(self.ptr_ty, closure_ptr, &[offset0], "fn_slot").unwrap()
                };
                let fn_ptr = self.builder.build_load(self.ptr_ty, fn_slot, "fn_ptr").unwrap().into_pointer_value();

                let mut param_types = Vec::new();
                for _ in 0..args.len() {
                    param_types.push(self.i64_ty.into());
                }
                let fn_ty = self.i64_ty.fn_type(&param_types, false);

                let av: Vec<BasicMetadataValueEnum<'ctx>> = args
                    .iter()
                    .map(|a| self.val(a).map(|v| v.into()))
                    .collect::<CompileResult<_>>()?;

                let rv = self.builder.build_indirect_call(fn_ty, fn_ptr, &av, "closure_call").unwrap();
                let v = rv
                    .try_as_basic_value()
                    .basic()
                    .map(|bv| bv.into_int_value())
                    .unwrap_or_else(|| self.nan.const_undefined());
                self.store(*dest, v);
            }

            MirInstr::TryEnter(jmp_buf_reg) => {
                let jmp_buf_ty = self.i8_ty.array_type(256);
                let jmp_buf_alloca = self.builder.build_alloca(jmp_buf_ty, "jmp_buf").unwrap();
                let jmp_buf_int = self.builder.build_ptr_to_int(jmp_buf_alloca, self.i64_ty, "jmp_buf_int").unwrap();
                self.store(*jmp_buf_reg, jmp_buf_int);

                let try_enter_fn = self.module.get_function("__bs_try_enter").unwrap();
                let ptr_val = self.builder.build_int_to_ptr(jmp_buf_int, self.ptr_ty, "jmp_buf_ptr").unwrap();
                self.builder.build_call(try_enter_fn, &[ptr_val.into()], "").unwrap();
            }

            MirInstr::SetJmp(dest_reg, jmp_buf_reg) => {
                let jmp_buf_int = self.val(&MirOperand::Reg(*jmp_buf_reg))?;
                let jmp_buf_ptr = self.builder.build_int_to_ptr(jmp_buf_int, self.ptr_ty, "jmp_buf_ptr").unwrap();
                let setjmp_fn = self.module.get_function("_setjmp").unwrap();
                
                let res = self.builder.build_call(setjmp_fn, &[jmp_buf_ptr.into()], "setjmp_res").unwrap()
                    .try_as_basic_value()
                    .basic()
                    .unwrap()
                    .into_int_value();
                
                // Compare setjmp result to 0
                let zero = self.i32_ty.const_int(0, false);
                let is_nonzero = self.builder.build_int_compare(IntPredicate::NE, res, zero, "is_nonzero").unwrap();
                
                // Box as Boolean: true if nonzero (longjmp return), false if zero (direct return)
                let boxed_bool = self.nan.box_bool(&self.builder, is_nonzero);
                self.store(*dest_reg, boxed_bool);
            }

            MirInstr::TryExit => {
                let try_exit_fn = self.module.get_function("__bs_try_exit").unwrap();
                self.builder.build_call(try_exit_fn, &[], "").unwrap();
            }

            MirInstr::Throw(val_operand) => {
                let val = self.val(val_operand)?;
                let throw_fn = self.module.get_function("__bs_throw").unwrap();
                self.builder.build_call(throw_fn, &[val.into()], "").unwrap();
                
                // __bs_throw is noreturn, but LLVM needs a terminator for the basic block to be valid.
                self.builder.build_unreachable().unwrap();
            }

            MirInstr::LoadGlobal(dest, name) => {
                let global = self.module.get_global(name).unwrap_or_else(|| {
                    let g = self.module.add_global(self.i64_ty, None, name);
                    g.set_initializer(&self.nan.const_undefined());
                    g
                });
                let loaded = self.builder.build_load(self.i64_ty, global.as_pointer_value(), "load_global").unwrap().into_int_value();
                self.store(*dest, loaded);
            }

            MirInstr::StoreGlobal(name, val_operand) => {
                let global = self.module.get_global(name).unwrap_or_else(|| {
                    let g = self.module.add_global(self.i64_ty, None, name);
                    g.set_initializer(&self.nan.const_undefined());
                    g
                });
                let val = self.val(val_operand)?;
                self.builder.build_store(global.as_pointer_value(), val).unwrap();
            }
        }
        Ok(())
    }

    // ── helpers ────────────────────────────────────────────────────────────

    fn emit_add(
        &mut self,
        dest: MirReg,
        l: &MirOperand,
        r: &MirOperand,
    ) -> CompileResult<()> {
        let lv = self.val(l)?;
        let rv = self.val(r)?;
        let func = self.module.get_function("__bs_add").ok_or_else(|| CompileError::Codegen {
            message: "runtime helper __bs_add not found".to_string(),
        })?;
        let call = self.builder.build_call(func, &[lv.into(), rv.into()], "add_call").unwrap();
        let res = call.try_as_basic_value().basic().unwrap().into_int_value();
        self.store(dest, res);
        Ok(())
    }

    fn emit_arith_f64(
        &mut self,
        dest: MirReg,
        l: &MirOperand,
        r: &MirOperand,
        op: &str,
    ) -> CompileResult<()> {
        let lv = self.val(l)?;
        let rv = self.val(r)?;
        let lf = self.nan.unbox_number(&self.builder, lv);
        let rf = self.nan.unbox_number(&self.builder, rv);
        let res = match op {
            "fadd" => self.builder.build_float_add(lf, rf, "add").unwrap(),
            "fsub" => self.builder.build_float_sub(lf, rf, "sub").unwrap(),
            "fmul" => self.builder.build_float_mul(lf, rf, "mul").unwrap(),
            "fdiv" => self.builder.build_float_div(lf, rf, "div").unwrap(),
            "frem" => self.builder.build_float_rem(lf, rf, "rem").unwrap(),
            _ => unreachable!(),
        };
        self.store(dest, self.nan.box_number(&self.builder, res));
        Ok(())
    }

    fn emit_cmp_f64(
        &mut self,
        dest: MirReg,
        l: &MirOperand,
        r: &MirOperand,
        pred: FloatPredicate,
    ) -> CompileResult<()> {
        let lv = self.val(l)?;
        let rv = self.val(r)?;
        let lf = self.nan.unbox_number(&self.builder, lv);
        let rf = self.nan.unbox_number(&self.builder, rv);
        let cmp = self.builder.build_float_compare(pred, lf, rf, "cmp").unwrap();
        self.store(dest, self.nan.box_bool(&self.builder, cmp));
        Ok(())
    }

    fn emit_eq(
        &mut self,
        dest: MirReg,
        l: &MirOperand,
        r: &MirOperand,
    ) -> CompileResult<()> {
        let lv = self.val(l)?;
        let rv = self.val(r)?;
        let func = self.module.get_function("__bs_strict_eq").ok_or_else(|| CompileError::Codegen {
            message: "runtime helper __bs_strict_eq not found".to_string(),
        })?;
        let call = self.builder.build_call(func, &[lv.into(), rv.into()], "eq_call").unwrap();
        let res = call.try_as_basic_value().basic().unwrap().into_int_value();
        self.store(dest, res);
        Ok(())
    }

    fn emit_ne(
        &mut self,
        dest: MirReg,
        l: &MirOperand,
        r: &MirOperand,
    ) -> CompileResult<()> {
        let lv = self.val(l)?;
        let rv = self.val(r)?;
        let func = self.module.get_function("__bs_strict_ne").ok_or_else(|| CompileError::Codegen {
            message: "runtime helper __bs_strict_ne not found".to_string(),
        })?;
        let call = self.builder.build_call(func, &[lv.into(), rv.into()], "ne_call").unwrap();
        let res = call.try_as_basic_value().basic().unwrap().into_int_value();
        self.store(dest, res);
        Ok(())
    }

    /// Resolve a `MirOperand` to an LLVM `i64` value.
    fn val(&mut self, op: &MirOperand) -> CompileResult<IntValue<'ctx>> {
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

    fn store(&self, reg: MirReg, v: IntValue<'ctx>) {
        if let Some(&a) = self.regs.get(&reg) {
            self.builder.build_store(a, v).unwrap();
        }
    }

    fn make_global_str(&mut self, s: &str) -> PointerValue<'ctx> {
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
