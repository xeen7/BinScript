warning: unused import: `CompileError`
 --> crates/hir/src/lower/expr/array.rs:3:19
  |
3 | use diagnostics::{CompileError, CompileResult};
  |                   ^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `CompileError`
 --> crates/hir/src/lower/expr/opt_chain.rs:3:19
  |
3 | use diagnostics::{CompileError, CompileResult};
  |                   ^^^^^^^^^^^^

warning: unreachable pattern
   --> crates/hir/src/lower/mod.rs:749:9
    |
749 |         _ => crate::types::UnaryOp::Neg,
    |         ^ no value can reach this
    |
note: multiple earlier patterns match some of the same values
   --> crates/hir/src/lower/mod.rs:749:9
    |
742 |         swc_core::ecma::ast::UnaryOp::Plus => crate::types::UnaryOp::Plus,
    |         ---------------------------------- matches some of the same values
743 |         swc_core::ecma::ast::UnaryOp::Minus => crate::types::UnaryOp::Neg,
    |         ----------------------------------- matches some of the same values
744 |         swc_core::ecma::ast::UnaryOp::Bang => crate::types::UnaryOp::Not,
    |         ---------------------------------- matches some of the same values
745 |         swc_core::ecma::ast::UnaryOp::Tilde => crate::types::UnaryOp::BitNot,
    |         ----------------------------------- matches some of the same values
...
749 |         _ => crate::types::UnaryOp::Neg,
    |         ^ ...and 3 other patterns collectively make this unreachable
    = note: `#[warn(unreachable_patterns)]` (part of `#[warn(unused)]`) on by default

warning: unnecessary `unsafe` block
   --> crates/codegen-llvm/src/codegen.rs:930:41
    |
930 |                     let state_idx_ptr = unsafe { self.builder.build_struct_gep(state_ty, state_ptr, 0, "state_idx_ptr").unwrap() };
    |                                         ^^^^^^ unnecessary `unsafe` block
    |
    = note: `#[warn(unused_unsafe)]` (part of `#[warn(unused)]`) on by default

warning: unnecessary `unsafe` block
    --> crates/codegen-llvm/src/codegen.rs:1191:32
     |
1191 | ...   let slot = unsafe { self.builder.build_struct_gep(state_ty, state_ptr, 2 + self.gen_num_args + *rid, "slot").unwrap() };
     |                  ^^^^^^ unnecessary `unsafe` block

warning: unnecessary `unsafe` block
    --> crates/codegen-llvm/src/codegen.rs:1195:37
     |
1195 |                 let state_idx_ptr = unsafe { self.builder.build_struct_gep(state_ty, state_ptr, 0, "state_idx_ptr").unwrap() };
     |                                     ^^^^^^ unnecessary `unsafe` block

warning: unnecessary `unsafe` block
    --> crates/codegen-llvm/src/codegen.rs:1211:32
     |
1211 | ...   let slot = unsafe { self.builder.build_struct_gep(state_ty, state_ptr, 2 + self.gen_num_args + *rid, "slot").unwrap() };
     |                  ^^^^^^ unnecessary `unsafe` block

warning: unnecessary `unsafe` block
    --> crates/codegen-llvm/src/codegen.rs:1229:41
     |
1229 |                     let state_idx_ptr = unsafe { self.builder.build_struct_gep(state_ty, state_ptr, 0, "state_idx_ptr").unwrap() };
     |                                         ^^^^^^ unnecessary `unsafe` block

; ModuleID = 'scratch/test_t17_bisect7.ts'
source_filename = "scratch/test_t17_bisect7.ts"

@.str.0 = unnamed_addr constant [3 x i8] c"%g\00"
@.str.1 = unnamed_addr constant [5 x i8] c"true\00"
@.str.2 = unnamed_addr constant [3 x i8] c"%s\00"
@.str.3 = unnamed_addr constant [6 x i8] c"false\00"
@.str.4 = unnamed_addr constant [6 x i8] c"%s {}\00"
@.str.5 = unnamed_addr constant [7 x i8] c"Object\00"
@.str.6 = unnamed_addr constant [5 x i8] c"null\00"
@.str.7 = unnamed_addr constant [10 x i8] c"undefined\00"
@.str.8 = unnamed_addr constant [11 x i8] c"[Function]\00"
@__bs_class_CaptureCell_vtable = constant { ptr, ptr, i64, i64, ptr } { ptr null, ptr @.str.9, i64 1, i64 1, ptr @__bs_class_CaptureCell_field_names }
@.str.9 = unnamed_addr constant [12 x i8] c"CaptureCell\00"
@.str.10 = unnamed_addr constant [6 x i8] c"value\00"
@__bs_class_CaptureCell_field_names = constant [1 x ptr] [ptr @.str.10]
@.str.11 = unnamed_addr constant [9 x i8] c"Assert [\00"
@.str.12 = unnamed_addr constant [13 x i8] c"]: expected \00"
@.str.13 = unnamed_addr constant [7 x i8] c", got \00"
@.str.14 = unnamed_addr constant [6 x i8] c" \E2\86\92 \00"
@.str.15 = unnamed_addr constant [5 x i8] c"PASS\00"
@.str.16 = unnamed_addr constant [5 x i8] c"FAIL\00"
@.str.17 = unnamed_addr constant [19 x i8] c"Assertion failed: \00"
@prop_str = private unnamed_addr constant [8 x i8] c"inStock\00", align 1
@prop_str.1 = private unnamed_addr constant [6 x i8] c"price\00", align 1
@prop_str.2 = private unnamed_addr constant [9 x i8] c"quantity\00", align 1
@prop_str.3 = private unnamed_addr constant [6 x i8] c"price\00", align 1
@prop_str.4 = private unnamed_addr constant [8 x i8] c"inStock\00", align 1
@prop_str.5 = private unnamed_addr constant [3 x i8] c"id\00", align 1
@.str.18 = unnamed_addr constant [3 x i8] c"p1\00"
@prop_str.6 = private unnamed_addr constant [5 x i8] c"name\00", align 1
@.str.19 = unnamed_addr constant [15 x i8] c"Wireless Mouse\00"
@prop_str.7 = private unnamed_addr constant [6 x i8] c"price\00", align 1
@prop_str.8 = private unnamed_addr constant [9 x i8] c"quantity\00", align 1
@prop_str.9 = private unnamed_addr constant [8 x i8] c"inStock\00", align 1
@prop_str.10 = private unnamed_addr constant [3 x i8] c"id\00", align 1
@.str.20 = unnamed_addr constant [3 x i8] c"p2\00"
@prop_str.11 = private unnamed_addr constant [5 x i8] c"name\00", align 1
@.str.21 = unnamed_addr constant [12 x i8] c"USB-C Cable\00"
@prop_str.12 = private unnamed_addr constant [6 x i8] c"price\00", align 1
@prop_str.13 = private unnamed_addr constant [9 x i8] c"quantity\00", align 1
@prop_str.14 = private unnamed_addr constant [8 x i8] c"inStock\00", align 1
@prop_str.15 = private unnamed_addr constant [3 x i8] c"id\00", align 1
@.str.22 = unnamed_addr constant [3 x i8] c"p3\00"
@prop_str.16 = private unnamed_addr constant [5 x i8] c"name\00", align 1
@.str.23 = unnamed_addr constant [20 x i8] c"Mechanical Keyboard\00"
@prop_str.17 = private unnamed_addr constant [6 x i8] c"price\00", align 1
@prop_str.18 = private unnamed_addr constant [9 x i8] c"quantity\00", align 1
@prop_str.19 = private unnamed_addr constant [8 x i8] c"inStock\00", align 1
@prop_str.20 = private unnamed_addr constant [3 x i8] c"id\00", align 1
@.str.24 = unnamed_addr constant [3 x i8] c"p4\00"
@prop_str.21 = private unnamed_addr constant [5 x i8] c"name\00", align 1
@.str.25 = unnamed_addr constant [11 x i8] c"4K Monitor\00"
@prop_str.22 = private unnamed_addr constant [6 x i8] c"price\00", align 1
@prop_str.23 = private unnamed_addr constant [9 x i8] c"quantity\00", align 1
@prop_str.24 = private unnamed_addr constant [8 x i8] c"inStock\00", align 1
@.str.26 = unnamed_addr constant [35 x i8] c"E-commerce grand total computation\00"
@.str.27 = unnamed_addr constant [38 x i8] c"Cart contains expensive items (> 250)\00"
@.str.28 = unnamed_addr constant [28 x i8] c"All cart items are in stock\00"
@prop_str.25 = private unnamed_addr constant [12 x i8] c"preferences\00", align 1
@prop_str.26 = private unnamed_addr constant [6 x i8] c"theme\00", align 1
@.str.29 = unnamed_addr constant [15 x i8] c"system-default\00"
@prop_str.27 = private unnamed_addr constant [8 x i8] c"contact\00", align 1
@prop_str.28 = private unnamed_addr constant [8 x i8] c"address\00", align 1
@prop_str.29 = private unnamed_addr constant [5 x i8] c"city\00", align 1
@.str.30 = unnamed_addr constant [13 x i8] c"Unknown City\00"
@prop_str.30 = private unnamed_addr constant [9 x i8] c"username\00", align 1
@.str.31 = unnamed_addr constant [9 x i8] c"coder123\00"
@prop_str.31 = private unnamed_addr constant [6 x i8] c"email\00", align 1
@.str.32 = unnamed_addr constant [21 x i8] c"coder123@example.com\00"
@prop_str.32 = private unnamed_addr constant [5 x i8] c"city\00", align 1
@.str.33 = unnamed_addr constant [14 x i8] c"San Francisco\00"
@prop_str.33 = private unnamed_addr constant [8 x i8] c"address\00", align 1
@prop_str.34 = private unnamed_addr constant [8 x i8] c"contact\00", align 1
@prop_str.35 = private unnamed_addr constant [6 x i8] c"theme\00", align 1
@.str.34 = unnamed_addr constant [10 x i8] c"dark-mode\00"
@prop_str.36 = private unnamed_addr constant [12 x i8] c"preferences\00", align 1
@prop_str.37 = private unnamed_addr constant [9 x i8] c"username\00", align 1
@.str.35 = unnamed_addr constant [11 x i8] c"guest_user\00"
@.str.36 = unnamed_addr constant [12 x i8] c"User1 theme\00"
@.str.37 = unnamed_addr constant [11 x i8] c"User1 city\00"
@.str.38 = unnamed_addr constant [12 x i8] c"User2 theme\00"
@.str.39 = unnamed_addr constant [11 x i8] c"User2 city\00"
@prop_str.38 = private unnamed_addr constant [9 x i8] c"username\00", align 1
@prop_str.39 = private unnamed_addr constant [12 x i8] c"preferences\00", align 1
@prop_str.40 = private unnamed_addr constant [12 x i8] c"preferences\00", align 1
@prop_str.41 = private unnamed_addr constant [9 x i8] c"fontSize\00", align 1
@prop_str.42 = private unnamed_addr constant [12 x i8] c"preferences\00", align 1
@.str.40 = unnamed_addr constant [22 x i8] c"Destructured username\00"
@.str.41 = unnamed_addr constant [22 x i8] c"Destructured fontSize\00"
@prop_str.43 = private unnamed_addr constant [12 x i8] c"preferences\00", align 1
@prop_str.44 = private unnamed_addr constant [9 x i8] c"fontSize\00", align 1
@prop_str.45 = private unnamed_addr constant [7 x i8] c"length\00", align 1
@.str.42 = unnamed_addr constant [2 x i8] c"$\00"
@.str.43 = unnamed_addr constant [15 x i8] c"Premium Coffee\00"
@.str.44 = unnamed_addr constant [10 x i8] c"Receipt: \00"
@.str.45 = unnamed_addr constant [3 x i8] c"x \00"
@.str.46 = unnamed_addr constant [5 x i8] c" at \00"
@.str.47 = unnamed_addr constant [6 x i8] c" each\00"
@.str.48 = unnamed_addr constant [41 x i8] c"Receipt: 3x Premium Coffee at $5.99 each\00"
@.str.49 = unnamed_addr constant [12 x i8] c"Invoice tag\00"
@.str.50 = unnamed_addr constant [14 x i8] c"=== START ===\00"
@.str.51 = unnamed_addr constant [10 x i8] c"Cart done\00"
@.str.52 = unnamed_addr constant [13 x i8] c"Profile done\00"
@.str.53 = unnamed_addr constant [17 x i8] c"=== ALL DONE ===\00"

declare i32 @printf(ptr, ...)

declare i32 @putchar(i32)

declare i64 @__bs_alloc(ptr, i64)

declare i64 @__bs_instanceof(i64, i64)

declare i64 @__bs_alloc_closure(i64)

declare i64 @__bs_alloc_generator(i64)

declare i64 @__bs_generator_next(i64, i64)

declare i64 @__bs_generator_is_done(i64)

declare void @__bs_drain_microtasks()

declare i64 @__bs_promise_new()

declare void @__bs_promise_resolve(i64, i64)

declare i64 @__bs_promise_then(i64, i64)

declare i64 @__bs_async_drive(i64)

declare i64 @__bs_promise_static_resolve(i64)

declare i64 @__bs_promise_all_2(i64, i64)

declare i64 @__bs_promise_race_2(i64, i64)

declare i64 @__bs_json_parse_lazy(ptr, i32)

declare i64 @__bs_json_tape_get(i64, ptr, i32)

declare i64 @__bs_prop_get(i64, ptr, i32)

declare i64 @__bs_prop_set(i64, ptr, i32, i64)

declare i64 @__bs_new_object()

declare i64 @__bs_index_get(i64, i64)

declare void @__bs_index_set(i64, i64, i64)

declare i64 @__bs_array_new()

declare i64 @__bs_array_push(i64, i64)

declare i64 @__bs_array_push_spread(i64, i64)

declare i64 @__bs_object_spread(i64, i64)

declare i64 @__bs_array_from(ptr, i32)

declare i64 @__bs_call_apply(i64, i64, i64)

declare i64 @__bs_vcall_apply(i64, i64, i64)

declare i64 @__bs_math_floor(i64)

declare i64 @__bs_math_ceil(i64)

declare i64 @__bs_math_round(i64)

declare i64 @__bs_math_abs(i64)

declare i64 @__bs_math_sqrt(i64)

declare i64 @__bs_math_pow(i64, i64)

declare i64 @__bs_math_min(i64, i64)

declare i64 @__bs_math_max(i64, i64)

declare i64 @__bs_math_log(i64)

declare i64 @__bs_math_log2(i64)

declare i64 @__bs_math_sin(i64)

declare i64 @__bs_math_cos(i64)

declare i64 @__bs_math_tan(i64)

declare i64 @__bs_math_random()

declare i64 @__bs_math_trunc(i64)

declare i64 @__bs_parseInt_1(i64)

declare i64 @__bs_parseInt_2(i64, i64)

declare i64 @__bs_parseFloat(i64)

declare i64 @__bs_isNaN(i64)

declare i64 @__bs_isFinite(i64)

declare i64 @__bs_number_isInteger(i64)

declare i64 @__bs_typeof(i64)

declare i64 @__bs_Object(i64)

declare i64 @__bs_Object_new(i64)

declare i64 @__bs_String(i64)

declare i64 @__bs_String_new(i64)

declare i64 @__bs_Number(i64)

declare i64 @__bs_Number_new(i64)

declare i64 @__bs_Boolean(i64)

declare i64 @__bs_Boolean_new(i64)

declare i64 @__bs_Date(i64)

declare i64 @__bs_Date_new(i64)

declare i64 @__bs_Array_new(i64)

declare i64 @__bs_RegExp_new(i64, i64)

declare i64 @__bs_Object_new_0()

declare i64 @__bs_Object_new_1(i64)

declare i64 @__bs_String_new_0()

declare i64 @__bs_String_new_1(i64)

declare i64 @__bs_Number_new_0()

declare i64 @__bs_Number_new_1(i64)

declare i64 @__bs_Boolean_new_0()

declare i64 @__bs_Boolean_new_1(i64)

declare i64 @__bs_Date_new_0()

declare i64 @__bs_Date_new_1(i64)

declare i64 @__bs_Date_new_n(i64, i64, i64, i64, i64, i64, i64)

declare i64 @__bs_object_keys(i64)

declare i64 @__bs_object_values(i64)

declare i64 @__bs_object_entries(i64)

declare i64 @__bs_object_assign(i64, i64)

declare i64 @__bs_object_create(i64)

declare i64 @__bs_object_getPrototypeOf(i64)

declare i64 @__bs_object_fromEntries(i64)

declare i64 @__bs_object_rest(i64, i64)

declare i64 @__bs_get_globalThis()

declare i64 @__bs_get_Symbol_global()

declare i64 @__bs_Symbol(i64)

declare i64 @__bs_Symbol_0()

declare i64 @__bs_Symbol_1(i64)

declare i64 @__bs_dynamic_import(i64)

declare i64 @__bs_encodeURI(i64)

declare i64 @__bs_decodeURI(i64)

declare i64 @__bs_encodeURIComponent(i64)

declare i64 @__bs_decodeURIComponent(i64)

declare i64 @__bs_URIError_new(i64)

declare i64 @__bs_string_fromCharCode(i64)

declare i64 @__bs_string_fromCodePoint(i64)

declare i64 @__bs_date_now()

declare i64 @__bs_strict_eq(i64, i64)

declare i64 @__bs_strict_ne(i64, i64)

declare i64 @__bs_add(i64, i64)

declare i64 @__bs_is_nullish(i64)

declare i64 @__bs_exp(i64, i64)

declare i64 @__bs_in(i64, i64)

declare i64 @__bs_delete_prop(i64, i64)

declare i64 @dummy_arr_0(i64, i64)

declare i64 @dummy_arr_1(i64, i64, i64)

declare i64 @dummy_arr_2(i64, i64, i64, i64)

declare i64 @__bs_call_push(i64, i64, i64)

declare i64 @__bs_call_pop(i64, i64)

declare i64 @__bs_call_slice(i64, i64, i64, i64)

declare i64 @__bs_call_indexOf(i64, i64, i64)

declare i64 @__bs_call_includes(i64, i64, i64)

declare i64 @__bs_call_next(i64, i64, i64)

declare i64 @__bs_call_join(i64, i64, i64)

declare i64 @__bs_call_reverse(i64, i64)

declare i64 @__bs_call_concat(i64, i64, i64)

declare i64 @__bs_call_fill(i64, i64, i64, i64, i64)

declare i64 @__bs_call_forEach(i64, i64, i64)

declare i64 @__bs_call_map(i64, i64, i64)

declare i64 @__bs_call_filter(i64, i64, i64)

declare i64 @__bs_call_find(i64, i64, i64)

declare i64 @__bs_call_findIndex(i64, i64, i64)

declare i64 @__bs_call_every(i64, i64, i64)

declare i64 @__bs_call_some(i64, i64, i64)

declare i64 @__bs_call_reduce(i64, i64, i64, i64)

declare i64 @__bs_call_charAt(i64, i64, i64)

declare i64 @__bs_call_charCodeAt(i64, i64, i64)

declare i64 @__bs_call_startsWith(i64, i64, i64)

declare i64 @__bs_call_endsWith(i64, i64, i64)

declare i64 @__bs_call_substring(i64, i64, i64, i64)

declare i64 @__bs_call_split(i64, i64, i64)

declare i64 @__bs_call_trim(i64, i64)

declare i64 @__bs_call_toUpperCase(i64, i64)

declare i64 @__bs_call_toLowerCase(i64, i64)

declare i64 @__bs_call_replace(i64, i64, i64, i64)

declare i64 @__bs_call_repeat(i64, i64, i64)

declare i64 @__bs_call_padStart(i64, i64, i64, i64)

declare i64 @__bs_call_padEnd(i64, i64, i64, i64)

declare i64 @__bs_call_getTime(i64, i64)

declare i64 @__bs_call_getFullYear(i64, i64)

declare i64 @__bs_call_getMonth(i64, i64)

declare i64 @__bs_call_getDate(i64, i64)

declare i64 @__bs_call_getHours(i64, i64)

declare i64 @__bs_call_getMinutes(i64, i64)

declare i64 @__bs_call_getSeconds(i64, i64)

declare i64 @__bs_call_toString(i64, i64)

declare i64 @__bs_call_valueOf(i64, i64)

declare i64 @__bs_fs_read_file_sync(i64)

declare void @__bs_fs_write_file_sync(i64, i64)

declare i64 @__bs_fs_exists_sync(i64)

declare i64 @__bs_path_join(i64, i64)

declare i64 @__bs_path_resolve(i64, i64)

declare i64 @__bs_os_platform()

declare i64 @__bs_os_arch()

declare void @__bs_shadow_push(ptr)

declare void @__bs_shadow_pop()

declare void @__bs_safepoint_poll()

; Function Attrs: returns_twice
declare i32 @_setjmp(ptr) #0

declare void @__bs_try_enter(ptr)

declare void @__bs_try_exit()

declare void @__bs_throw(i64)

declare i64 @__bs_get_and_clear_exception()

declare i64 @__bs_Error_new(i64)

declare i64 @__bs_TypeError_new(i64)

declare i64 @__bs_RangeError_new(i64)

declare i64 @__bs_ReferenceError_new(i64)

declare i64 @__bs_SyntaxError_new(i64)

define void @__bs_console_log_1(i64 %0) {
entry:
  %top16 = lshr i64 %0, 48
  %tagged = icmp uge i64 %top16, 65521
  br i1 %tagged, label %check_tag, label %print_num

print_num:                                        ; preds = %entry
  %f = bitcast i64 %0 to double
  %1 = call i32 (ptr, ...) @printf(ptr @.str.0, double %f)
  br label %done

check_tag:                                        ; preds = %entry
  switch i64 %top16, label %print_undef [
    i64 65521, label %print_undef
    i64 65522, label %print_null
    i64 65523, label %print_false
    i64 65524, label %print_true
    i64 65526, label %print_obj
    i64 65527, label %print_str
    i64 65528, label %print_symbol
    i64 65529, label %print_closure
  ]

print_true:                                       ; preds = %check_tag
  %2 = call i32 (ptr, ...) @printf(ptr @.str.2, ptr @.str.1)
  br label %done

print_false:                                      ; preds = %check_tag
  %3 = call i32 (ptr, ...) @printf(ptr @.str.2, ptr @.str.3)
  br label %done

print_str:                                        ; preds = %check_tag
  %payload = and i64 %0, 281474976710655
  %sptr = inttoptr i64 %payload to ptr
  %4 = call i32 (ptr, ...) @printf(ptr @.str.2, ptr %sptr)
  br label %done

print_obj:                                        ; preds = %check_tag
  %payload1 = and i64 %0, 281474976710655
  %obj_ptr = inttoptr i64 %payload1 to ptr
  %vtable_ptr = load ptr, ptr %obj_ptr, align 8
  %vtable_addr = ptrtoint ptr %vtable_ptr to i64
  %has_vtable = icmp ne i64 %vtable_addr, 0
  br i1 %has_vtable, label %load_name, label %default_name

print_null:                                       ; preds = %check_tag
  %5 = call i32 (ptr, ...) @printf(ptr @.str.2, ptr @.str.6)
  br label %done

print_undef:                                      ; preds = %check_tag, %check_tag
  %6 = call i32 (ptr, ...) @printf(ptr @.str.2, ptr @.str.7)
  br label %done

print_closure:                                    ; preds = %check_tag
  %7 = call i32 (ptr, ...) @printf(ptr @.str.2, ptr @.str.8)
  br label %done

print_symbol:                                     ; preds = %check_tag
  %sym_str = call i64 @__bs_String(i64 %0)
  %sym_payload = and i64 %sym_str, 281474976710655
  %sym_sptr = inttoptr i64 %sym_payload to ptr
  %8 = call i32 (ptr, ...) @printf(ptr @.str.2, ptr %sym_sptr)
  br label %done

done:                                             ; preds = %print_symbol, %print_closure, %print_undef, %print_null, %default_name, %load_name, %print_str, %print_false, %print_true, %print_num
  %9 = call i32 @putchar(i32 10)
  ret void

load_name:                                        ; preds = %print_obj
  %name_ptr_ptr = getelementptr ptr, ptr %vtable_ptr, i32 1
  %name_ptr = load ptr, ptr %name_ptr_ptr, align 8
  %10 = call i32 (ptr, ...) @printf(ptr @.str.4, ptr %name_ptr)
  br label %done

default_name:                                     ; preds = %print_obj
  %11 = call i32 (ptr, ...) @printf(ptr @.str.4, ptr @.str.5)
  br label %done
}

define i64 @__bs_assertEqual(i64 %0, i64 %1, i64 %2, i64 %3) {
bb0:
  %regs_array = alloca [18 x i64], align 8
  %4 = ptrtoint ptr %regs_array to i64
  %regs_ptr = inttoptr i64 %4 to ptr
  %r0 = getelementptr i64, ptr %regs_array, i32 0
  store i64 -4222124650659840, ptr %r0, align 4
  %r1 = getelementptr i64, ptr %regs_array, i32 1
  store i64 -4222124650659840, ptr %r1, align 4
  %r2 = getelementptr i64, ptr %regs_array, i32 2
  store i64 -4222124650659840, ptr %r2, align 4
  %r3 = getelementptr i64, ptr %regs_array, i32 3
  store i64 -4222124650659840, ptr %r3, align 4
  %r4 = getelementptr i64, ptr %regs_array, i32 4
  store i64 -4222124650659840, ptr %r4, align 4
  %r5 = getelementptr i64, ptr %regs_array, i32 5
  store i64 -4222124650659840, ptr %r5, align 4
  %r6 = getelementptr i64, ptr %regs_array, i32 6
  store i64 -4222124650659840, ptr %r6, align 4
  %r7 = getelementptr i64, ptr %regs_array, i32 7
  store i64 -4222124650659840, ptr %r7, align 4
  %r8 = getelementptr i64, ptr %regs_array, i32 8
  store i64 -4222124650659840, ptr %r8, align 4
  %r9 = getelementptr i64, ptr %regs_array, i32 9
  store i64 -4222124650659840, ptr %r9, align 4
  %r10 = getelementptr i64, ptr %regs_array, i32 10
  store i64 -4222124650659840, ptr %r10, align 4
  %r11 = getelementptr i64, ptr %regs_array, i32 11
  store i64 -4222124650659840, ptr %r11, align 4
  %r12 = getelementptr i64, ptr %regs_array, i32 12
  store i64 -4222124650659840, ptr %r12, align 4
  %r13 = getelementptr i64, ptr %regs_array, i32 13
  store i64 -4222124650659840, ptr %r13, align 4
  %r14 = getelementptr i64, ptr %regs_array, i32 14
  store i64 -4222124650659840, ptr %r14, align 4
  %r15 = getelementptr i64, ptr %regs_array, i32 15
  store i64 -4222124650659840, ptr %r15, align 4
  %r16 = getelementptr i64, ptr %regs_array, i32 16
  store i64 -4222124650659840, ptr %r16, align 4
  %r17 = getelementptr i64, ptr %regs_array, i32 17
  store i64 -4222124650659840, ptr %r17, align 4
  %shadow_frame = alloca { ptr, i32, i32, ptr }, align 8
  %num_roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 1
  store i32 18, ptr %num_roots_ptr, align 4
  %roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 3
  store ptr %regs_ptr, ptr %roots_ptr, align 8
  call void @__bs_shadow_push(ptr %shadow_frame)
  store i64 %0, ptr %r0, align 4
  store i64 %1, ptr %r1, align 4
  store i64 %2, ptr %r2, align 4
  store i64 %3, ptr %r3, align 4
  %r18 = load i64, ptr %r1, align 4
  %r29 = load i64, ptr %r2, align 4
  %eq_call = call i64 @__bs_strict_eq(i64 %r18, i64 %r29)
  store i64 %eq_call, ptr %r5, align 4
  %r510 = load i64, ptr %r5, align 4
  store i64 %r510, ptr %r4, align 4
  %box_str = or i64 ptrtoint (ptr @.str.11 to i64), -2533274790395904
  %r311 = load i64, ptr %r3, align 4
  %add_call = call i64 @__bs_add(i64 %box_str, i64 %r311)
  store i64 %add_call, ptr %r6, align 4
  %r612 = load i64, ptr %r6, align 4
  %box_str13 = or i64 ptrtoint (ptr @.str.12 to i64), -2533274790395904
  %add_call14 = call i64 @__bs_add(i64 %r612, i64 %box_str13)
  store i64 %add_call14, ptr %r7, align 4
  %r715 = load i64, ptr %r7, align 4
  %r216 = load i64, ptr %r2, align 4
  %add_call17 = call i64 @__bs_add(i64 %r715, i64 %r216)
  store i64 %add_call17, ptr %r8, align 4
  %r818 = load i64, ptr %r8, align 4
  %box_str19 = or i64 ptrtoint (ptr @.str.13 to i64), -2533274790395904
  %add_call20 = call i64 @__bs_add(i64 %r818, i64 %box_str19)
  store i64 %add_call20, ptr %r9, align 4
  %r921 = load i64, ptr %r9, align 4
  %r122 = load i64, ptr %r1, align 4
  %add_call23 = call i64 @__bs_add(i64 %r921, i64 %r122)
  store i64 %add_call23, ptr %r10, align 4
  %r1024 = load i64, ptr %r10, align 4
  %box_str25 = or i64 ptrtoint (ptr @.str.14 to i64), -2533274790395904
  %add_call26 = call i64 @__bs_add(i64 %r1024, i64 %box_str25)
  store i64 %add_call26, ptr %r11, align 4
  %r427 = load i64, ptr %r4, align 4
  %t16 = lshr i64 %r427, 48
  %is_tagged = icmp uge i64 %t16, 65521
  %tag_truthy = icmp uge i64 %t16, 65524
  %asf64 = bitcast i64 %r427 to double
  %num_truthy = fcmp one double %asf64, 0.000000e+00
  %truthy = select i1 %is_tagged, i1 %tag_truthy, i1 %num_truthy
  br i1 %truthy, label %bb1, label %bb2

bb1:                                              ; preds = %bb0
  %box_str28 = or i64 ptrtoint (ptr @.str.15 to i64), -2533274790395904
  store i64 %box_str28, ptr %r12, align 4
  br label %bb3

bb2:                                              ; preds = %bb0
  %box_str29 = or i64 ptrtoint (ptr @.str.16 to i64), -2533274790395904
  store i64 %box_str29, ptr %r12, align 4
  br label %bb3

bb3:                                              ; preds = %bb2, %bb1
  %r1130 = load i64, ptr %r11, align 4
  %r1231 = load i64, ptr %r12, align 4
  %add_call32 = call i64 @__bs_add(i64 %r1130, i64 %r1231)
  store i64 %add_call32, ptr %r13, align 4
  %r1333 = load i64, ptr %r13, align 4
  call void @__bs_console_log_1(i64 %r1333)
  store i64 -4222124650659840, ptr %r14, align 4
  %r434 = load i64, ptr %r4, align 4
  %t1635 = lshr i64 %r434, 48
  %is_tagged36 = icmp uge i64 %t1635, 65521
  %tag_truthy37 = icmp uge i64 %t1635, 65524
  %asf6438 = bitcast i64 %r434 to double
  %num_truthy39 = fcmp one double %asf6438, 0.000000e+00
  %truthy40 = select i1 %is_tagged36, i1 %tag_truthy37, i1 %num_truthy39
  %not = xor i1 %truthy40, true
  %box_bool = select i1 %not, i64 -3377699720527872, i64 -3659174697238528
  store i64 %box_bool, ptr %r15, align 4
  %r1541 = load i64, ptr %r15, align 4
  %t1642 = lshr i64 %r1541, 48
  %is_tagged43 = icmp uge i64 %t1642, 65521
  %tag_truthy44 = icmp uge i64 %t1642, 65524
  %asf6445 = bitcast i64 %r1541 to double
  %num_truthy46 = fcmp one double %asf6445, 0.000000e+00
  %truthy47 = select i1 %is_tagged43, i1 %tag_truthy44, i1 %num_truthy46
  br i1 %truthy47, label %bb4, label %bb5

bb4:                                              ; preds = %bb3
  %box_str48 = or i64 ptrtoint (ptr @.str.17 to i64), -2533274790395904
  %r349 = load i64, ptr %r3, align 4
  %add_call50 = call i64 @__bs_add(i64 %box_str48, i64 %r349)
  store i64 %add_call50, ptr %r16, align 4
  %r1651 = load i64, ptr %r16, align 4
  %call = call i64 @__bs_Error_new(i64 %r1651)
  store i64 %call, ptr %r17, align 4
  %r1752 = load i64, ptr %r17, align 4
  call void @__bs_throw(i64 %r1752)
  unreachable

bb5:                                              ; preds = %bb3
  br label %bb6

bb6:                                              ; preds = %bb5
  call void @__bs_shadow_pop()
  ret i64 -4222124650659840
}

define i64 @__bs_closure_3(i64 %0, i64 %1) {
bb0:
  %regs_array = alloca [3 x i64], align 8
  %2 = ptrtoint ptr %regs_array to i64
  %regs_ptr = inttoptr i64 %2 to ptr
  %r0 = getelementptr i64, ptr %regs_array, i32 0
  store i64 -4222124650659840, ptr %r0, align 4
  %r1 = getelementptr i64, ptr %regs_array, i32 1
  store i64 -4222124650659840, ptr %r1, align 4
  %r2 = getelementptr i64, ptr %regs_array, i32 2
  store i64 -4222124650659840, ptr %r2, align 4
  %shadow_frame = alloca { ptr, i32, i32, ptr }, align 8
  %num_roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 1
  store i32 3, ptr %num_roots_ptr, align 4
  %roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 3
  store ptr %regs_ptr, ptr %roots_ptr, align 8
  call void @__bs_shadow_push(ptr %shadow_frame)
  store i64 %0, ptr %r0, align 4
  store i64 %1, ptr %r1, align 4
  %r11 = load i64, ptr %r1, align 4
  %prop_get = call i64 @__bs_prop_get(i64 %r11, ptr @prop_str, i32 7)
  store i64 %prop_get, ptr %r2, align 4
  %r22 = load i64, ptr %r2, align 4
  call void @__bs_shadow_pop()
  ret i64 %r22
}

define i64 @__bs_closure_4(i64 %0, i64 %1) {
bb0:
  %regs_array = alloca [5 x i64], align 8
  %2 = ptrtoint ptr %regs_array to i64
  %regs_ptr = inttoptr i64 %2 to ptr
  %r0 = getelementptr i64, ptr %regs_array, i32 0
  store i64 -4222124650659840, ptr %r0, align 4
  %r1 = getelementptr i64, ptr %regs_array, i32 1
  store i64 -4222124650659840, ptr %r1, align 4
  %r2 = getelementptr i64, ptr %regs_array, i32 2
  store i64 -4222124650659840, ptr %r2, align 4
  %r3 = getelementptr i64, ptr %regs_array, i32 3
  store i64 -4222124650659840, ptr %r3, align 4
  %r4 = getelementptr i64, ptr %regs_array, i32 4
  store i64 -4222124650659840, ptr %r4, align 4
  %shadow_frame = alloca { ptr, i32, i32, ptr }, align 8
  %num_roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 1
  store i32 5, ptr %num_roots_ptr, align 4
  %roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 3
  store ptr %regs_ptr, ptr %roots_ptr, align 8
  call void @__bs_shadow_push(ptr %shadow_frame)
  store i64 %0, ptr %r0, align 4
  store i64 %1, ptr %r1, align 4
  %r11 = load i64, ptr %r1, align 4
  %prop_get = call i64 @__bs_prop_get(i64 %r11, ptr @prop_str.1, i32 5)
  store i64 %prop_get, ptr %r2, align 4
  %r12 = load i64, ptr %r1, align 4
  %prop_get3 = call i64 @__bs_prop_get(i64 %r12, ptr @prop_str.2, i32 8)
  store i64 %prop_get3, ptr %r3, align 4
  %r24 = load i64, ptr %r2, align 4
  %r35 = load i64, ptr %r3, align 4
  %unbox_num = bitcast i64 %r24 to double
  %unbox_num6 = bitcast i64 %r35 to double
  %mul = fmul double %unbox_num, %unbox_num6
  %box_num = bitcast double %mul to i64
  store i64 %box_num, ptr %r4, align 4
  %r47 = load i64, ptr %r4, align 4
  call void @__bs_shadow_pop()
  ret i64 %r47
}

define i64 @__bs_closure_5(i64 %0, i64 %1, i64 %2) {
bb0:
  %regs_array = alloca [4 x i64], align 8
  %3 = ptrtoint ptr %regs_array to i64
  %regs_ptr = inttoptr i64 %3 to ptr
  %r0 = getelementptr i64, ptr %regs_array, i32 0
  store i64 -4222124650659840, ptr %r0, align 4
  %r1 = getelementptr i64, ptr %regs_array, i32 1
  store i64 -4222124650659840, ptr %r1, align 4
  %r2 = getelementptr i64, ptr %regs_array, i32 2
  store i64 -4222124650659840, ptr %r2, align 4
  %r3 = getelementptr i64, ptr %regs_array, i32 3
  store i64 -4222124650659840, ptr %r3, align 4
  %shadow_frame = alloca { ptr, i32, i32, ptr }, align 8
  %num_roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 1
  store i32 4, ptr %num_roots_ptr, align 4
  %roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 3
  store ptr %regs_ptr, ptr %roots_ptr, align 8
  call void @__bs_shadow_push(ptr %shadow_frame)
  store i64 %0, ptr %r0, align 4
  store i64 %1, ptr %r1, align 4
  store i64 %2, ptr %r2, align 4
  %r11 = load i64, ptr %r1, align 4
  %r22 = load i64, ptr %r2, align 4
  %add_call = call i64 @__bs_add(i64 %r11, i64 %r22)
  store i64 %add_call, ptr %r3, align 4
  %r33 = load i64, ptr %r3, align 4
  call void @__bs_shadow_pop()
  ret i64 %r33
}

define i64 @__bs_calculateCartTotal(i64 %0, i64 %1) {
bb0:
  %regs_array = alloca [11 x i64], align 8
  %2 = ptrtoint ptr %regs_array to i64
  %regs_ptr = inttoptr i64 %2 to ptr
  %r0 = getelementptr i64, ptr %regs_array, i32 0
  store i64 -4222124650659840, ptr %r0, align 4
  %r1 = getelementptr i64, ptr %regs_array, i32 1
  store i64 -4222124650659840, ptr %r1, align 4
  %r2 = getelementptr i64, ptr %regs_array, i32 2
  store i64 -4222124650659840, ptr %r2, align 4
  %r3 = getelementptr i64, ptr %regs_array, i32 3
  store i64 -4222124650659840, ptr %r3, align 4
  %r4 = getelementptr i64, ptr %regs_array, i32 4
  store i64 -4222124650659840, ptr %r4, align 4
  %r5 = getelementptr i64, ptr %regs_array, i32 5
  store i64 -4222124650659840, ptr %r5, align 4
  %r6 = getelementptr i64, ptr %regs_array, i32 6
  store i64 -4222124650659840, ptr %r6, align 4
  %r7 = getelementptr i64, ptr %regs_array, i32 7
  store i64 -4222124650659840, ptr %r7, align 4
  %r8 = getelementptr i64, ptr %regs_array, i32 8
  store i64 -4222124650659840, ptr %r8, align 4
  %r9 = getelementptr i64, ptr %regs_array, i32 9
  store i64 -4222124650659840, ptr %r9, align 4
  %r10 = getelementptr i64, ptr %regs_array, i32 10
  store i64 -4222124650659840, ptr %r10, align 4
  %shadow_frame = alloca { ptr, i32, i32, ptr }, align 8
  %num_roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 1
  store i32 11, ptr %num_roots_ptr, align 4
  %roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 3
  store ptr %regs_ptr, ptr %roots_ptr, align 8
  call void @__bs_shadow_push(ptr %shadow_frame)
  store i64 %0, ptr %r0, align 4
  store i64 %1, ptr %r1, align 4
  %alloc_closure = call i64 @__bs_alloc_closure(i64 16)
  %payload = and i64 %alloc_closure, 281474976710655
  %closure_ptr = inttoptr i64 %payload to ptr
  %fn_slot = getelementptr ptr, ptr %closure_ptr, i32 0
  store ptr @__bs_closure_3, ptr %fn_slot, align 8
  %unused_slot = getelementptr i64, ptr %closure_ptr, i32 1
  store i64 -4222124650659840, ptr %unused_slot, align 4
  store i64 %alloc_closure, ptr %r3, align 4
  %r11 = load i64, ptr %r1, align 4
  %r32 = load i64, ptr %r3, align 4
  %call = call i64 @__bs_call_filter(i64 %r11, i64 %r32, i64 -4616189618054758400)
  store i64 %call, ptr %r4, align 4
  %r43 = load i64, ptr %r4, align 4
  store i64 %r43, ptr %r2, align 4
  %alloc_closure4 = call i64 @__bs_alloc_closure(i64 16)
  %payload5 = and i64 %alloc_closure4, 281474976710655
  %closure_ptr6 = inttoptr i64 %payload5 to ptr
  %fn_slot7 = getelementptr ptr, ptr %closure_ptr6, i32 0
  store ptr @__bs_closure_4, ptr %fn_slot7, align 8
  %unused_slot8 = getelementptr i64, ptr %closure_ptr6, i32 1
  store i64 -4222124650659840, ptr %unused_slot8, align 4
  store i64 %alloc_closure4, ptr %r6, align 4
  %r29 = load i64, ptr %r2, align 4
  %r610 = load i64, ptr %r6, align 4
  %call11 = call i64 @__bs_call_map(i64 %r29, i64 %r610, i64 -4616189618054758400)
  store i64 %call11, ptr %r7, align 4
  %r712 = load i64, ptr %r7, align 4
  store i64 %r712, ptr %r5, align 4
  %alloc_closure13 = call i64 @__bs_alloc_closure(i64 16)
  %payload14 = and i64 %alloc_closure13, 281474976710655
  %closure_ptr15 = inttoptr i64 %payload14 to ptr
  %fn_slot16 = getelementptr ptr, ptr %closure_ptr15, i32 0
  store ptr @__bs_closure_5, ptr %fn_slot16, align 8
  %unused_slot17 = getelementptr i64, ptr %closure_ptr15, i32 1
  store i64 -4222124650659840, ptr %unused_slot17, align 4
  store i64 %alloc_closure13, ptr %r9, align 4
  %r518 = load i64, ptr %r5, align 4
  %r919 = load i64, ptr %r9, align 4
  %call20 = call i64 @__bs_call_reduce(i64 %r518, i64 %r919, i64 0, i64 -4616189618054758400)
  store i64 %call20, ptr %r10, align 4
  %r1021 = load i64, ptr %r10, align 4
  store i64 %r1021, ptr %r8, align 4
  %r822 = load i64, ptr %r8, align 4
  call void @__bs_shadow_pop()
  ret i64 %r822
}

define i64 @__bs_closure_7(i64 %0, i64 %1) {
bb0:
  %regs_array = alloca [4 x i64], align 8
  %2 = ptrtoint ptr %regs_array to i64
  %regs_ptr = inttoptr i64 %2 to ptr
  %r0 = getelementptr i64, ptr %regs_array, i32 0
  store i64 -4222124650659840, ptr %r0, align 4
  %r1 = getelementptr i64, ptr %regs_array, i32 1
  store i64 -4222124650659840, ptr %r1, align 4
  %r2 = getelementptr i64, ptr %regs_array, i32 2
  store i64 -4222124650659840, ptr %r2, align 4
  %r3 = getelementptr i64, ptr %regs_array, i32 3
  store i64 -4222124650659840, ptr %r3, align 4
  %shadow_frame = alloca { ptr, i32, i32, ptr }, align 8
  %num_roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 1
  store i32 4, ptr %num_roots_ptr, align 4
  %roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 3
  store ptr %regs_ptr, ptr %roots_ptr, align 8
  call void @__bs_shadow_push(ptr %shadow_frame)
  store i64 %0, ptr %r0, align 4
  store i64 %1, ptr %r1, align 4
  %r11 = load i64, ptr %r1, align 4
  %prop_get = call i64 @__bs_prop_get(i64 %r11, ptr @prop_str.3, i32 5)
  store i64 %prop_get, ptr %r2, align 4
  %r22 = load i64, ptr %r2, align 4
  %unbox_num = bitcast i64 %r22 to double
  %cmp = fcmp ogt double %unbox_num, 2.500000e+02
  %box_bool = select i1 %cmp, i64 -3377699720527872, i64 -3659174697238528
  store i64 %box_bool, ptr %r3, align 4
  %r33 = load i64, ptr %r3, align 4
  call void @__bs_shadow_pop()
  ret i64 %r33
}

define i64 @__bs_closure_8(i64 %0, i64 %1) {
bb0:
  %regs_array = alloca [3 x i64], align 8
  %2 = ptrtoint ptr %regs_array to i64
  %regs_ptr = inttoptr i64 %2 to ptr
  %r0 = getelementptr i64, ptr %regs_array, i32 0
  store i64 -4222124650659840, ptr %r0, align 4
  %r1 = getelementptr i64, ptr %regs_array, i32 1
  store i64 -4222124650659840, ptr %r1, align 4
  %r2 = getelementptr i64, ptr %regs_array, i32 2
  store i64 -4222124650659840, ptr %r2, align 4
  %shadow_frame = alloca { ptr, i32, i32, ptr }, align 8
  %num_roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 1
  store i32 3, ptr %num_roots_ptr, align 4
  %roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 3
  store ptr %regs_ptr, ptr %roots_ptr, align 8
  call void @__bs_shadow_push(ptr %shadow_frame)
  store i64 %0, ptr %r0, align 4
  store i64 %1, ptr %r1, align 4
  %r11 = load i64, ptr %r1, align 4
  %prop_get = call i64 @__bs_prop_get(i64 %r11, ptr @prop_str.4, i32 7)
  store i64 %prop_get, ptr %r2, align 4
  %r22 = load i64, ptr %r2, align 4
  call void @__bs_shadow_pop()
  ret i64 %r22
}

define i64 @__bs_runCartTests(i64 %0) {
bb0:
  %regs_array = alloca [26 x i64], align 8
  %1 = ptrtoint ptr %regs_array to i64
  %regs_ptr = inttoptr i64 %1 to ptr
  %r0 = getelementptr i64, ptr %regs_array, i32 0
  store i64 -4222124650659840, ptr %r0, align 4
  %r1 = getelementptr i64, ptr %regs_array, i32 1
  store i64 -4222124650659840, ptr %r1, align 4
  %r2 = getelementptr i64, ptr %regs_array, i32 2
  store i64 -4222124650659840, ptr %r2, align 4
  %r3 = getelementptr i64, ptr %regs_array, i32 3
  store i64 -4222124650659840, ptr %r3, align 4
  %r4 = getelementptr i64, ptr %regs_array, i32 4
  store i64 -4222124650659840, ptr %r4, align 4
  %r5 = getelementptr i64, ptr %regs_array, i32 5
  store i64 -4222124650659840, ptr %r5, align 4
  %r6 = getelementptr i64, ptr %regs_array, i32 6
  store i64 -4222124650659840, ptr %r6, align 4
  %r7 = getelementptr i64, ptr %regs_array, i32 7
  store i64 -4222124650659840, ptr %r7, align 4
  %r8 = getelementptr i64, ptr %regs_array, i32 8
  store i64 -4222124650659840, ptr %r8, align 4
  %r9 = getelementptr i64, ptr %regs_array, i32 9
  store i64 -4222124650659840, ptr %r9, align 4
  %r10 = getelementptr i64, ptr %regs_array, i32 10
  store i64 -4222124650659840, ptr %r10, align 4
  %r11 = getelementptr i64, ptr %regs_array, i32 11
  store i64 -4222124650659840, ptr %r11, align 4
  %r12 = getelementptr i64, ptr %regs_array, i32 12
  store i64 -4222124650659840, ptr %r12, align 4
  %r13 = getelementptr i64, ptr %regs_array, i32 13
  store i64 -4222124650659840, ptr %r13, align 4
  %r14 = getelementptr i64, ptr %regs_array, i32 14
  store i64 -4222124650659840, ptr %r14, align 4
  %r15 = getelementptr i64, ptr %regs_array, i32 15
  store i64 -4222124650659840, ptr %r15, align 4
  %r16 = getelementptr i64, ptr %regs_array, i32 16
  store i64 -4222124650659840, ptr %r16, align 4
  %r17 = getelementptr i64, ptr %regs_array, i32 17
  store i64 -4222124650659840, ptr %r17, align 4
  %r18 = getelementptr i64, ptr %regs_array, i32 18
  store i64 -4222124650659840, ptr %r18, align 4
  %r19 = getelementptr i64, ptr %regs_array, i32 19
  store i64 -4222124650659840, ptr %r19, align 4
  %r20 = getelementptr i64, ptr %regs_array, i32 20
  store i64 -4222124650659840, ptr %r20, align 4
  %r21 = getelementptr i64, ptr %regs_array, i32 21
  store i64 -4222124650659840, ptr %r21, align 4
  %r22 = getelementptr i64, ptr %regs_array, i32 22
  store i64 -4222124650659840, ptr %r22, align 4
  %r23 = getelementptr i64, ptr %regs_array, i32 23
  store i64 -4222124650659840, ptr %r23, align 4
  %r24 = getelementptr i64, ptr %regs_array, i32 24
  store i64 -4222124650659840, ptr %r24, align 4
  %r25 = getelementptr i64, ptr %regs_array, i32 25
  store i64 -4222124650659840, ptr %r25, align 4
  %shadow_frame = alloca { ptr, i32, i32, ptr }, align 8
  %num_roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 1
  store i32 26, ptr %num_roots_ptr, align 4
  %roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 3
  store ptr %regs_ptr, ptr %roots_ptr, align 8
  call void @__bs_shadow_push(ptr %shadow_frame)
  store i64 %0, ptr %r0, align 4
  %call = call i64 @__bs_array_new()
  store i64 %call, ptr %r2, align 4
  %call1 = call i64 @__bs_new_object()
  store i64 %call1, ptr %r3, align 4
  %r32 = load i64, ptr %r3, align 4
  store i64 %r32, ptr %r4, align 4
  %r43 = load i64, ptr %r4, align 4
  %box_str = or i64 ptrtoint (ptr @.str.18 to i64), -2533274790395904
  %prop_set = call i64 @__bs_prop_set(i64 %r43, ptr @prop_str.5, i32 2, i64 %box_str)
  %r44 = load i64, ptr %r4, align 4
  %box_str5 = or i64 ptrtoint (ptr @.str.19 to i64), -2533274790395904
  %prop_set6 = call i64 @__bs_prop_set(i64 %r44, ptr @prop_str.6, i32 4, i64 %box_str5)
  %r47 = load i64, ptr %r4, align 4
  %prop_set8 = call i64 @__bs_prop_set(i64 %r47, ptr @prop_str.7, i32 5, i64 4627730092099895296)
  %r49 = load i64, ptr %r4, align 4
  %prop_set10 = call i64 @__bs_prop_set(i64 %r49, ptr @prop_str.8, i32 8, i64 4611686018427387904)
  %r411 = load i64, ptr %r4, align 4
  %prop_set12 = call i64 @__bs_prop_set(i64 %r411, ptr @prop_str.9, i32 7, i64 -3377699720527872)
  %r213 = load i64, ptr %r2, align 4
  %r414 = load i64, ptr %r4, align 4
  %call15 = call i64 @__bs_array_push(i64 %r213, i64 %r414)
  store i64 %call15, ptr %r5, align 4
  %call16 = call i64 @__bs_new_object()
  store i64 %call16, ptr %r6, align 4
  %r617 = load i64, ptr %r6, align 4
  store i64 %r617, ptr %r7, align 4
  %r718 = load i64, ptr %r7, align 4
  %box_str19 = or i64 ptrtoint (ptr @.str.20 to i64), -2533274790395904
  %prop_set20 = call i64 @__bs_prop_set(i64 %r718, ptr @prop_str.10, i32 2, i64 %box_str19)
  %r721 = load i64, ptr %r7, align 4
  %box_str22 = or i64 ptrtoint (ptr @.str.21 to i64), -2533274790395904
  %prop_set23 = call i64 @__bs_prop_set(i64 %r721, ptr @prop_str.11, i32 4, i64 %box_str22)
  %r724 = load i64, ptr %r7, align 4
  %prop_set25 = call i64 @__bs_prop_set(i64 %r724, ptr @prop_str.12, i32 5, i64 4623226492472524800)
  %r726 = load i64, ptr %r7, align 4
  %prop_set27 = call i64 @__bs_prop_set(i64 %r726, ptr @prop_str.13, i32 8, i64 4613937818241073152)
  %r728 = load i64, ptr %r7, align 4
  %prop_set29 = call i64 @__bs_prop_set(i64 %r728, ptr @prop_str.14, i32 7, i64 -3377699720527872)
  %r230 = load i64, ptr %r2, align 4
  %r731 = load i64, ptr %r7, align 4
  %call32 = call i64 @__bs_array_push(i64 %r230, i64 %r731)
  store i64 %call32, ptr %r8, align 4
  %call33 = call i64 @__bs_new_object()
  store i64 %call33, ptr %r9, align 4
  %r934 = load i64, ptr %r9, align 4
  store i64 %r934, ptr %r10, align 4
  %r1035 = load i64, ptr %r10, align 4
  %box_str36 = or i64 ptrtoint (ptr @.str.22 to i64), -2533274790395904
  %prop_set37 = call i64 @__bs_prop_set(i64 %r1035, ptr @prop_str.15, i32 2, i64 %box_str36)
  %r1038 = load i64, ptr %r10, align 4
  %box_str39 = or i64 ptrtoint (ptr @.str.23 to i64), -2533274790395904
  %prop_set40 = call i64 @__bs_prop_set(i64 %r1038, ptr @prop_str.16, i32 4, i64 %box_str39)
  %r1041 = load i64, ptr %r10, align 4
  %prop_set42 = call i64 @__bs_prop_set(i64 %r1041, ptr @prop_str.17, i32 5, i64 4636666922610458624)
  %r1043 = load i64, ptr %r10, align 4
  %prop_set44 = call i64 @__bs_prop_set(i64 %r1043, ptr @prop_str.18, i32 8, i64 4607182418800017408)
  %r1045 = load i64, ptr %r10, align 4
  %prop_set46 = call i64 @__bs_prop_set(i64 %r1045, ptr @prop_str.19, i32 7, i64 -3659174697238528)
  %r247 = load i64, ptr %r2, align 4
  %r1048 = load i64, ptr %r10, align 4
  %call49 = call i64 @__bs_array_push(i64 %r247, i64 %r1048)
  store i64 %call49, ptr %r11, align 4
  %call50 = call i64 @__bs_new_object()
  store i64 %call50, ptr %r12, align 4
  %r1251 = load i64, ptr %r12, align 4
  store i64 %r1251, ptr %r13, align 4
  %r1352 = load i64, ptr %r13, align 4
  %box_str53 = or i64 ptrtoint (ptr @.str.24 to i64), -2533274790395904
  %prop_set54 = call i64 @__bs_prop_set(i64 %r1352, ptr @prop_str.20, i32 2, i64 %box_str53)
  %r1355 = load i64, ptr %r13, align 4
  %box_str56 = or i64 ptrtoint (ptr @.str.25 to i64), -2533274790395904
  %prop_set57 = call i64 @__bs_prop_set(i64 %r1355, ptr @prop_str.21, i32 4, i64 %box_str56)
  %r1358 = load i64, ptr %r13, align 4
  %prop_set59 = call i64 @__bs_prop_set(i64 %r1358, ptr @prop_str.22, i32 5, i64 4643967679818891264)
  %r1360 = load i64, ptr %r13, align 4
  %prop_set61 = call i64 @__bs_prop_set(i64 %r1360, ptr @prop_str.23, i32 8, i64 4607182418800017408)
  %r1362 = load i64, ptr %r13, align 4
  %prop_set63 = call i64 @__bs_prop_set(i64 %r1362, ptr @prop_str.24, i32 7, i64 -3377699720527872)
  %r264 = load i64, ptr %r2, align 4
  %r1365 = load i64, ptr %r13, align 4
  %call66 = call i64 @__bs_array_push(i64 %r264, i64 %r1365)
  store i64 %call66, ptr %r14, align 4
  %r267 = load i64, ptr %r2, align 4
  store i64 %r267, ptr %r1, align 4
  %r168 = load i64, ptr %r1, align 4
  %call69 = call i64 @__bs_calculateCartTotal(i64 -4222124650659840, i64 %r168)
  store i64 %call69, ptr %r16, align 4
  %r1670 = load i64, ptr %r16, align 4
  store i64 %r1670, ptr %r15, align 4
  %r1571 = load i64, ptr %r15, align 4
  %box_str72 = or i64 ptrtoint (ptr @.str.26 to i64), -2533274790395904
  %call73 = call i64 @__bs_assertEqual(i64 -4222124650659840, i64 %r1571, i64 4645506996097777664, i64 %box_str72)
  store i64 %call73, ptr %r17, align 4
  %alloc_closure = call i64 @__bs_alloc_closure(i64 16)
  %payload = and i64 %alloc_closure, 281474976710655
  %closure_ptr = inttoptr i64 %payload to ptr
  %fn_slot = getelementptr ptr, ptr %closure_ptr, i32 0
  store ptr @__bs_closure_7, ptr %fn_slot, align 8
  %unused_slot = getelementptr i64, ptr %closure_ptr, i32 1
  store i64 -4222124650659840, ptr %unused_slot, align 4
  store i64 %alloc_closure, ptr %r19, align 4
  %r174 = load i64, ptr %r1, align 4
  %r1975 = load i64, ptr %r19, align 4
  %call76 = call i64 @__bs_call_some(i64 %r174, i64 %r1975, i64 -4616189618054758400)
  store i64 %call76, ptr %r20, align 4
  %r2077 = load i64, ptr %r20, align 4
  store i64 %r2077, ptr %r18, align 4
  %alloc_closure78 = call i64 @__bs_alloc_closure(i64 16)
  %payload79 = and i64 %alloc_closure78, 281474976710655
  %closure_ptr80 = inttoptr i64 %payload79 to ptr
  %fn_slot81 = getelementptr ptr, ptr %closure_ptr80, i32 0
  store ptr @__bs_closure_8, ptr %fn_slot81, align 8
  %unused_slot82 = getelementptr i64, ptr %closure_ptr80, i32 1
  store i64 -4222124650659840, ptr %unused_slot82, align 4
  store i64 %alloc_closure78, ptr %r22, align 4
  %r183 = load i64, ptr %r1, align 4
  %r2284 = load i64, ptr %r22, align 4
  %call85 = call i64 @__bs_call_every(i64 %r183, i64 %r2284, i64 -4616189618054758400)
  store i64 %call85, ptr %r23, align 4
  %r2386 = load i64, ptr %r23, align 4
  store i64 %r2386, ptr %r21, align 4
  %r1887 = load i64, ptr %r18, align 4
  %box_str88 = or i64 ptrtoint (ptr @.str.27 to i64), -2533274790395904
  %call89 = call i64 @__bs_assertEqual(i64 -4222124650659840, i64 %r1887, i64 -3377699720527872, i64 %box_str88)
  store i64 %call89, ptr %r24, align 4
  %r2190 = load i64, ptr %r21, align 4
  %box_str91 = or i64 ptrtoint (ptr @.str.28 to i64), -2533274790395904
  %call92 = call i64 @__bs_assertEqual(i64 -4222124650659840, i64 %r2190, i64 -3659174697238528, i64 %box_str91)
  store i64 %call92, ptr %r25, align 4
  call void @__bs_shadow_pop()
  ret i64 -4222124650659840
}

define i64 @__bs_getUserTheme(i64 %0, i64 %1) {
bb0:
  %regs_array = alloca [11 x i64], align 8
  %2 = ptrtoint ptr %regs_array to i64
  %regs_ptr = inttoptr i64 %2 to ptr
  %r0 = getelementptr i64, ptr %regs_array, i32 0
  store i64 -4222124650659840, ptr %r0, align 4
  %r1 = getelementptr i64, ptr %regs_array, i32 1
  store i64 -4222124650659840, ptr %r1, align 4
  %r2 = getelementptr i64, ptr %regs_array, i32 2
  store i64 -4222124650659840, ptr %r2, align 4
  %r3 = getelementptr i64, ptr %regs_array, i32 3
  store i64 -4222124650659840, ptr %r3, align 4
  %r4 = getelementptr i64, ptr %regs_array, i32 4
  store i64 -4222124650659840, ptr %r4, align 4
  %r5 = getelementptr i64, ptr %regs_array, i32 5
  store i64 -4222124650659840, ptr %r5, align 4
  %r6 = getelementptr i64, ptr %regs_array, i32 6
  store i64 -4222124650659840, ptr %r6, align 4
  %r7 = getelementptr i64, ptr %regs_array, i32 7
  store i64 -4222124650659840, ptr %r7, align 4
  %r8 = getelementptr i64, ptr %regs_array, i32 8
  store i64 -4222124650659840, ptr %r8, align 4
  %r9 = getelementptr i64, ptr %regs_array, i32 9
  store i64 -4222124650659840, ptr %r9, align 4
  %r10 = getelementptr i64, ptr %regs_array, i32 10
  store i64 -4222124650659840, ptr %r10, align 4
  %shadow_frame = alloca { ptr, i32, i32, ptr }, align 8
  %num_roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 1
  store i32 11, ptr %num_roots_ptr, align 4
  %roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 3
  store ptr %regs_ptr, ptr %roots_ptr, align 8
  call void @__bs_shadow_push(ptr %shadow_frame)
  store i64 %0, ptr %r0, align 4
  store i64 %1, ptr %r1, align 4
  %r11 = load i64, ptr %r1, align 4
  %prop_get = call i64 @__bs_prop_get(i64 %r11, ptr @prop_str.25, i32 11)
  store i64 %prop_get, ptr %r2, align 4
  %r22 = load i64, ptr %r2, align 4
  store i64 %r22, ptr %r3, align 4
  %r33 = load i64, ptr %r3, align 4
  %eq_call = call i64 @__bs_strict_eq(i64 %r33, i64 -3940649673949184)
  store i64 %eq_call, ptr %r4, align 4
  %r44 = load i64, ptr %r4, align 4
  store i64 %r44, ptr %r5, align 4
  %r45 = load i64, ptr %r4, align 4
  %t16 = lshr i64 %r45, 48
  %is_tagged = icmp uge i64 %t16, 65521
  %tag_truthy = icmp uge i64 %t16, 65524
  %asf64 = bitcast i64 %r45 to double
  %num_truthy = fcmp one double %asf64, 0.000000e+00
  %truthy = select i1 %is_tagged, i1 %tag_truthy, i1 %num_truthy
  br i1 %truthy, label %bb2, label %bb1

bb1:                                              ; preds = %bb0
  %r36 = load i64, ptr %r3, align 4
  %eq_call7 = call i64 @__bs_strict_eq(i64 %r36, i64 -4222124650659840)
  store i64 %eq_call7, ptr %r6, align 4
  %r68 = load i64, ptr %r6, align 4
  store i64 %r68, ptr %r5, align 4
  br label %bb2

bb2:                                              ; preds = %bb1, %bb0
  %r59 = load i64, ptr %r5, align 4
  %t1610 = lshr i64 %r59, 48
  %is_tagged11 = icmp uge i64 %t1610, 65521
  %tag_truthy12 = icmp uge i64 %t1610, 65524
  %asf6413 = bitcast i64 %r59 to double
  %num_truthy14 = fcmp one double %asf6413, 0.000000e+00
  %truthy15 = select i1 %is_tagged11, i1 %tag_truthy12, i1 %num_truthy14
  br i1 %truthy15, label %bb3, label %bb4

bb3:                                              ; preds = %bb2
  store i64 -4222124650659840, ptr %r7, align 4
  br label %bb5

bb4:                                              ; preds = %bb2
  %r316 = load i64, ptr %r3, align 4
  %prop_get17 = call i64 @__bs_prop_get(i64 %r316, ptr @prop_str.26, i32 5)
  store i64 %prop_get17, ptr %r8, align 4
  %r818 = load i64, ptr %r8, align 4
  store i64 %r818, ptr %r3, align 4
  %r819 = load i64, ptr %r8, align 4
  store i64 %r819, ptr %r7, align 4
  br label %bb5

bb5:                                              ; preds = %bb4, %bb3
  %r720 = load i64, ptr %r7, align 4
  store i64 %r720, ptr %r9, align 4
  %r721 = load i64, ptr %r7, align 4
  %call = call i64 @__bs_is_nullish(i64 %r721)
  store i64 %call, ptr %r10, align 4
  %r1022 = load i64, ptr %r10, align 4
  %t1623 = lshr i64 %r1022, 48
  %is_tagged24 = icmp uge i64 %t1623, 65521
  %tag_truthy25 = icmp uge i64 %t1623, 65524
  %asf6426 = bitcast i64 %r1022 to double
  %num_truthy27 = fcmp one double %asf6426, 0.000000e+00
  %truthy28 = select i1 %is_tagged24, i1 %tag_truthy25, i1 %num_truthy27
  br i1 %truthy28, label %bb6, label %bb7

bb6:                                              ; preds = %bb5
  %box_str = or i64 ptrtoint (ptr @.str.29 to i64), -2533274790395904
  store i64 %box_str, ptr %r9, align 4
  br label %bb7

bb7:                                              ; preds = %bb6, %bb5
  %r929 = load i64, ptr %r9, align 4
  call void @__bs_shadow_pop()
  ret i64 %r929
}

define i64 @__bs_getUserCity(i64 %0, i64 %1) {
bb0:
  %regs_array = alloca [16 x i64], align 8
  %2 = ptrtoint ptr %regs_array to i64
  %regs_ptr = inttoptr i64 %2 to ptr
  %r0 = getelementptr i64, ptr %regs_array, i32 0
  store i64 -4222124650659840, ptr %r0, align 4
  %r1 = getelementptr i64, ptr %regs_array, i32 1
  store i64 -4222124650659840, ptr %r1, align 4
  %r2 = getelementptr i64, ptr %regs_array, i32 2
  store i64 -4222124650659840, ptr %r2, align 4
  %r3 = getelementptr i64, ptr %regs_array, i32 3
  store i64 -4222124650659840, ptr %r3, align 4
  %r4 = getelementptr i64, ptr %regs_array, i32 4
  store i64 -4222124650659840, ptr %r4, align 4
  %r5 = getelementptr i64, ptr %regs_array, i32 5
  store i64 -4222124650659840, ptr %r5, align 4
  %r6 = getelementptr i64, ptr %regs_array, i32 6
  store i64 -4222124650659840, ptr %r6, align 4
  %r7 = getelementptr i64, ptr %regs_array, i32 7
  store i64 -4222124650659840, ptr %r7, align 4
  %r8 = getelementptr i64, ptr %regs_array, i32 8
  store i64 -4222124650659840, ptr %r8, align 4
  %r9 = getelementptr i64, ptr %regs_array, i32 9
  store i64 -4222124650659840, ptr %r9, align 4
  %r10 = getelementptr i64, ptr %regs_array, i32 10
  store i64 -4222124650659840, ptr %r10, align 4
  %r11 = getelementptr i64, ptr %regs_array, i32 11
  store i64 -4222124650659840, ptr %r11, align 4
  %r12 = getelementptr i64, ptr %regs_array, i32 12
  store i64 -4222124650659840, ptr %r12, align 4
  %r13 = getelementptr i64, ptr %regs_array, i32 13
  store i64 -4222124650659840, ptr %r13, align 4
  %r14 = getelementptr i64, ptr %regs_array, i32 14
  store i64 -4222124650659840, ptr %r14, align 4
  %r15 = getelementptr i64, ptr %regs_array, i32 15
  store i64 -4222124650659840, ptr %r15, align 4
  %shadow_frame = alloca { ptr, i32, i32, ptr }, align 8
  %num_roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 1
  store i32 16, ptr %num_roots_ptr, align 4
  %roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 3
  store ptr %regs_ptr, ptr %roots_ptr, align 8
  call void @__bs_shadow_push(ptr %shadow_frame)
  store i64 %0, ptr %r0, align 4
  store i64 %1, ptr %r1, align 4
  %r16 = load i64, ptr %r1, align 4
  %prop_get = call i64 @__bs_prop_get(i64 %r16, ptr @prop_str.27, i32 7)
  store i64 %prop_get, ptr %r2, align 4
  %r27 = load i64, ptr %r2, align 4
  store i64 %r27, ptr %r3, align 4
  %r38 = load i64, ptr %r3, align 4
  %eq_call = call i64 @__bs_strict_eq(i64 %r38, i64 -3940649673949184)
  store i64 %eq_call, ptr %r4, align 4
  %r49 = load i64, ptr %r4, align 4
  store i64 %r49, ptr %r5, align 4
  %r410 = load i64, ptr %r4, align 4
  %t16 = lshr i64 %r410, 48
  %is_tagged = icmp uge i64 %t16, 65521
  %tag_truthy = icmp uge i64 %t16, 65524
  %asf64 = bitcast i64 %r410 to double
  %num_truthy = fcmp one double %asf64, 0.000000e+00
  %truthy = select i1 %is_tagged, i1 %tag_truthy, i1 %num_truthy
  br i1 %truthy, label %bb2, label %bb1

bb1:                                              ; preds = %bb0
  %r311 = load i64, ptr %r3, align 4
  %eq_call12 = call i64 @__bs_strict_eq(i64 %r311, i64 -4222124650659840)
  store i64 %eq_call12, ptr %r6, align 4
  %r613 = load i64, ptr %r6, align 4
  store i64 %r613, ptr %r5, align 4
  br label %bb2

bb2:                                              ; preds = %bb1, %bb0
  %r514 = load i64, ptr %r5, align 4
  %t1615 = lshr i64 %r514, 48
  %is_tagged16 = icmp uge i64 %t1615, 65521
  %tag_truthy17 = icmp uge i64 %t1615, 65524
  %asf6418 = bitcast i64 %r514 to double
  %num_truthy19 = fcmp one double %asf6418, 0.000000e+00
  %truthy20 = select i1 %is_tagged16, i1 %tag_truthy17, i1 %num_truthy19
  br i1 %truthy20, label %bb3, label %bb4

bb3:                                              ; preds = %bb2
  store i64 -4222124650659840, ptr %r7, align 4
  br label %bb5

bb4:                                              ; preds = %bb2
  %r321 = load i64, ptr %r3, align 4
  %prop_get22 = call i64 @__bs_prop_get(i64 %r321, ptr @prop_str.28, i32 7)
  store i64 %prop_get22, ptr %r8, align 4
  %r823 = load i64, ptr %r8, align 4
  store i64 %r823, ptr %r3, align 4
  %r324 = load i64, ptr %r3, align 4
  %eq_call25 = call i64 @__bs_strict_eq(i64 %r324, i64 -3940649673949184)
  store i64 %eq_call25, ptr %r9, align 4
  %r926 = load i64, ptr %r9, align 4
  store i64 %r926, ptr %r10, align 4
  %r927 = load i64, ptr %r9, align 4
  %t1628 = lshr i64 %r927, 48
  %is_tagged29 = icmp uge i64 %t1628, 65521
  %tag_truthy30 = icmp uge i64 %t1628, 65524
  %asf6431 = bitcast i64 %r927 to double
  %num_truthy32 = fcmp one double %asf6431, 0.000000e+00
  %truthy33 = select i1 %is_tagged29, i1 %tag_truthy30, i1 %num_truthy32
  br i1 %truthy33, label %bb7, label %bb6

bb5:                                              ; preds = %bb10, %bb3
  %r734 = load i64, ptr %r7, align 4
  store i64 %r734, ptr %r14, align 4
  %r735 = load i64, ptr %r7, align 4
  %call = call i64 @__bs_is_nullish(i64 %r735)
  store i64 %call, ptr %r15, align 4
  %r1536 = load i64, ptr %r15, align 4
  %t1637 = lshr i64 %r1536, 48
  %is_tagged38 = icmp uge i64 %t1637, 65521
  %tag_truthy39 = icmp uge i64 %t1637, 65524
  %asf6440 = bitcast i64 %r1536 to double
  %num_truthy41 = fcmp one double %asf6440, 0.000000e+00
  %truthy42 = select i1 %is_tagged38, i1 %tag_truthy39, i1 %num_truthy41
  br i1 %truthy42, label %bb11, label %bb12

bb6:                                              ; preds = %bb4
  %r343 = load i64, ptr %r3, align 4
  %eq_call44 = call i64 @__bs_strict_eq(i64 %r343, i64 -4222124650659840)
  store i64 %eq_call44, ptr %r11, align 4
  %r1145 = load i64, ptr %r11, align 4
  store i64 %r1145, ptr %r10, align 4
  br label %bb7

bb7:                                              ; preds = %bb6, %bb4
  %r1046 = load i64, ptr %r10, align 4
  %t1647 = lshr i64 %r1046, 48
  %is_tagged48 = icmp uge i64 %t1647, 65521
  %tag_truthy49 = icmp uge i64 %t1647, 65524
  %asf6450 = bitcast i64 %r1046 to double
  %num_truthy51 = fcmp one double %asf6450, 0.000000e+00
  %truthy52 = select i1 %is_tagged48, i1 %tag_truthy49, i1 %num_truthy51
  br i1 %truthy52, label %bb8, label %bb9

bb8:                                              ; preds = %bb7
  store i64 -4222124650659840, ptr %r12, align 4
  br label %bb10

bb9:                                              ; preds = %bb7
  %r353 = load i64, ptr %r3, align 4
  %prop_get54 = call i64 @__bs_prop_get(i64 %r353, ptr @prop_str.29, i32 4)
  store i64 %prop_get54, ptr %r13, align 4
  %r1355 = load i64, ptr %r13, align 4
  store i64 %r1355, ptr %r3, align 4
  %r1356 = load i64, ptr %r13, align 4
  store i64 %r1356, ptr %r12, align 4
  br label %bb10

bb10:                                             ; preds = %bb9, %bb8
  %r1257 = load i64, ptr %r12, align 4
  store i64 %r1257, ptr %r7, align 4
  call void @__bs_safepoint_poll()
  br label %bb5

bb11:                                             ; preds = %bb5
  %box_str = or i64 ptrtoint (ptr @.str.30 to i64), -2533274790395904
  store i64 %box_str, ptr %r14, align 4
  br label %bb12

bb12:                                             ; preds = %bb11, %bb5
  %r1458 = load i64, ptr %r14, align 4
  call void @__bs_shadow_pop()
  ret i64 %r1458
}

define i64 @__bs_runProfileTests(i64 %0) {
bb0:
  %regs_array = alloca [42 x i64], align 8
  %1 = ptrtoint ptr %regs_array to i64
  %regs_ptr = inttoptr i64 %1 to ptr
  %r0 = getelementptr i64, ptr %regs_array, i32 0
  store i64 -4222124650659840, ptr %r0, align 4
  %r1 = getelementptr i64, ptr %regs_array, i32 1
  store i64 -4222124650659840, ptr %r1, align 4
  %r2 = getelementptr i64, ptr %regs_array, i32 2
  store i64 -4222124650659840, ptr %r2, align 4
  %r3 = getelementptr i64, ptr %regs_array, i32 3
  store i64 -4222124650659840, ptr %r3, align 4
  %r4 = getelementptr i64, ptr %regs_array, i32 4
  store i64 -4222124650659840, ptr %r4, align 4
  %r5 = getelementptr i64, ptr %regs_array, i32 5
  store i64 -4222124650659840, ptr %r5, align 4
  %r6 = getelementptr i64, ptr %regs_array, i32 6
  store i64 -4222124650659840, ptr %r6, align 4
  %r7 = getelementptr i64, ptr %regs_array, i32 7
  store i64 -4222124650659840, ptr %r7, align 4
  %r8 = getelementptr i64, ptr %regs_array, i32 8
  store i64 -4222124650659840, ptr %r8, align 4
  %r9 = getelementptr i64, ptr %regs_array, i32 9
  store i64 -4222124650659840, ptr %r9, align 4
  %r10 = getelementptr i64, ptr %regs_array, i32 10
  store i64 -4222124650659840, ptr %r10, align 4
  %r11 = getelementptr i64, ptr %regs_array, i32 11
  store i64 -4222124650659840, ptr %r11, align 4
  %r12 = getelementptr i64, ptr %regs_array, i32 12
  store i64 -4222124650659840, ptr %r12, align 4
  %r13 = getelementptr i64, ptr %regs_array, i32 13
  store i64 -4222124650659840, ptr %r13, align 4
  %r14 = getelementptr i64, ptr %regs_array, i32 14
  store i64 -4222124650659840, ptr %r14, align 4
  %r15 = getelementptr i64, ptr %regs_array, i32 15
  store i64 -4222124650659840, ptr %r15, align 4
  %r16 = getelementptr i64, ptr %regs_array, i32 16
  store i64 -4222124650659840, ptr %r16, align 4
  %r17 = getelementptr i64, ptr %regs_array, i32 17
  store i64 -4222124650659840, ptr %r17, align 4
  %r18 = getelementptr i64, ptr %regs_array, i32 18
  store i64 -4222124650659840, ptr %r18, align 4
  %r19 = getelementptr i64, ptr %regs_array, i32 19
  store i64 -4222124650659840, ptr %r19, align 4
  %r20 = getelementptr i64, ptr %regs_array, i32 20
  store i64 -4222124650659840, ptr %r20, align 4
  %r21 = getelementptr i64, ptr %regs_array, i32 21
  store i64 -4222124650659840, ptr %r21, align 4
  %r22 = getelementptr i64, ptr %regs_array, i32 22
  store i64 -4222124650659840, ptr %r22, align 4
  %r23 = getelementptr i64, ptr %regs_array, i32 23
  store i64 -4222124650659840, ptr %r23, align 4
  %r24 = getelementptr i64, ptr %regs_array, i32 24
  store i64 -4222124650659840, ptr %r24, align 4
  %r25 = getelementptr i64, ptr %regs_array, i32 25
  store i64 -4222124650659840, ptr %r25, align 4
  %r26 = getelementptr i64, ptr %regs_array, i32 26
  store i64 -4222124650659840, ptr %r26, align 4
  %r27 = getelementptr i64, ptr %regs_array, i32 27
  store i64 -4222124650659840, ptr %r27, align 4
  %r28 = getelementptr i64, ptr %regs_array, i32 28
  store i64 -4222124650659840, ptr %r28, align 4
  %r29 = getelementptr i64, ptr %regs_array, i32 29
  store i64 -4222124650659840, ptr %r29, align 4
  %r30 = getelementptr i64, ptr %regs_array, i32 30
  store i64 -4222124650659840, ptr %r30, align 4
  %r31 = getelementptr i64, ptr %regs_array, i32 31
  store i64 -4222124650659840, ptr %r31, align 4
  %r32 = getelementptr i64, ptr %regs_array, i32 32
  store i64 -4222124650659840, ptr %r32, align 4
  %r33 = getelementptr i64, ptr %regs_array, i32 33
  store i64 -4222124650659840, ptr %r33, align 4
  %r34 = getelementptr i64, ptr %regs_array, i32 34
  store i64 -4222124650659840, ptr %r34, align 4
  %r35 = getelementptr i64, ptr %regs_array, i32 35
  store i64 -4222124650659840, ptr %r35, align 4
  %r36 = getelementptr i64, ptr %regs_array, i32 36
  store i64 -4222124650659840, ptr %r36, align 4
  %r37 = getelementptr i64, ptr %regs_array, i32 37
  store i64 -4222124650659840, ptr %r37, align 4
  %r38 = getelementptr i64, ptr %regs_array, i32 38
  store i64 -4222124650659840, ptr %r38, align 4
  %r39 = getelementptr i64, ptr %regs_array, i32 39
  store i64 -4222124650659840, ptr %r39, align 4
  %r40 = getelementptr i64, ptr %regs_array, i32 40
  store i64 -4222124650659840, ptr %r40, align 4
  %r41 = getelementptr i64, ptr %regs_array, i32 41
  store i64 -4222124650659840, ptr %r41, align 4
  %shadow_frame = alloca { ptr, i32, i32, ptr }, align 8
  %num_roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 1
  store i32 42, ptr %num_roots_ptr, align 4
  %roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 3
  store ptr %regs_ptr, ptr %roots_ptr, align 8
  call void @__bs_shadow_push(ptr %shadow_frame)
  store i64 %0, ptr %r0, align 4
  %call = call i64 @__bs_new_object()
  store i64 %call, ptr %r2, align 4
  %r210 = load i64, ptr %r2, align 4
  store i64 %r210, ptr %r3, align 4
  %r311 = load i64, ptr %r3, align 4
  %box_str = or i64 ptrtoint (ptr @.str.31 to i64), -2533274790395904
  %prop_set = call i64 @__bs_prop_set(i64 %r311, ptr @prop_str.30, i32 8, i64 %box_str)
  %call12 = call i64 @__bs_new_object()
  store i64 %call12, ptr %r4, align 4
  %r413 = load i64, ptr %r4, align 4
  store i64 %r413, ptr %r5, align 4
  %r514 = load i64, ptr %r5, align 4
  %box_str15 = or i64 ptrtoint (ptr @.str.32 to i64), -2533274790395904
  %prop_set16 = call i64 @__bs_prop_set(i64 %r514, ptr @prop_str.31, i32 5, i64 %box_str15)
  %call17 = call i64 @__bs_new_object()
  store i64 %call17, ptr %r6, align 4
  %r618 = load i64, ptr %r6, align 4
  store i64 %r618, ptr %r7, align 4
  %r719 = load i64, ptr %r7, align 4
  %box_str20 = or i64 ptrtoint (ptr @.str.33 to i64), -2533274790395904
  %prop_set21 = call i64 @__bs_prop_set(i64 %r719, ptr @prop_str.32, i32 4, i64 %box_str20)
  %r522 = load i64, ptr %r5, align 4
  %r723 = load i64, ptr %r7, align 4
  %prop_set24 = call i64 @__bs_prop_set(i64 %r522, ptr @prop_str.33, i32 7, i64 %r723)
  %r325 = load i64, ptr %r3, align 4
  %r526 = load i64, ptr %r5, align 4
  %prop_set27 = call i64 @__bs_prop_set(i64 %r325, ptr @prop_str.34, i32 7, i64 %r526)
  %call28 = call i64 @__bs_new_object()
  store i64 %call28, ptr %r8, align 4
  %r829 = load i64, ptr %r8, align 4
  store i64 %r829, ptr %r9, align 4
  %r930 = load i64, ptr %r9, align 4
  %box_str31 = or i64 ptrtoint (ptr @.str.34 to i64), -2533274790395904
  %prop_set32 = call i64 @__bs_prop_set(i64 %r930, ptr @prop_str.35, i32 5, i64 %box_str31)
  %r333 = load i64, ptr %r3, align 4
  %r934 = load i64, ptr %r9, align 4
  %prop_set35 = call i64 @__bs_prop_set(i64 %r333, ptr @prop_str.36, i32 11, i64 %r934)
  %r336 = load i64, ptr %r3, align 4
  store i64 %r336, ptr %r1, align 4
  %call37 = call i64 @__bs_new_object()
  store i64 %call37, ptr %r11, align 4
  %r1138 = load i64, ptr %r11, align 4
  store i64 %r1138, ptr %r12, align 4
  %r1239 = load i64, ptr %r12, align 4
  %box_str40 = or i64 ptrtoint (ptr @.str.35 to i64), -2533274790395904
  %prop_set41 = call i64 @__bs_prop_set(i64 %r1239, ptr @prop_str.37, i32 8, i64 %box_str40)
  %r1242 = load i64, ptr %r12, align 4
  store i64 %r1242, ptr %r10, align 4
  %r143 = load i64, ptr %r1, align 4
  %call44 = call i64 @__bs_getUserTheme(i64 -4222124650659840, i64 %r143)
  store i64 %call44, ptr %r13, align 4
  %r1345 = load i64, ptr %r13, align 4
  %box_str46 = or i64 ptrtoint (ptr @.str.34 to i64), -2533274790395904
  %box_str47 = or i64 ptrtoint (ptr @.str.36 to i64), -2533274790395904
  %call48 = call i64 @__bs_assertEqual(i64 -4222124650659840, i64 %r1345, i64 %box_str46, i64 %box_str47)
  store i64 %call48, ptr %r14, align 4
  %r149 = load i64, ptr %r1, align 4
  %call50 = call i64 @__bs_getUserCity(i64 -4222124650659840, i64 %r149)
  store i64 %call50, ptr %r15, align 4
  %r1551 = load i64, ptr %r15, align 4
  %box_str52 = or i64 ptrtoint (ptr @.str.33 to i64), -2533274790395904
  %box_str53 = or i64 ptrtoint (ptr @.str.37 to i64), -2533274790395904
  %call54 = call i64 @__bs_assertEqual(i64 -4222124650659840, i64 %r1551, i64 %box_str52, i64 %box_str53)
  store i64 %call54, ptr %r16, align 4
  %r1055 = load i64, ptr %r10, align 4
  %call56 = call i64 @__bs_getUserTheme(i64 -4222124650659840, i64 %r1055)
  store i64 %call56, ptr %r17, align 4
  %r1757 = load i64, ptr %r17, align 4
  %box_str58 = or i64 ptrtoint (ptr @.str.29 to i64), -2533274790395904
  %box_str59 = or i64 ptrtoint (ptr @.str.38 to i64), -2533274790395904
  %call60 = call i64 @__bs_assertEqual(i64 -4222124650659840, i64 %r1757, i64 %box_str58, i64 %box_str59)
  store i64 %call60, ptr %r18, align 4
  %r1061 = load i64, ptr %r10, align 4
  %call62 = call i64 @__bs_getUserCity(i64 -4222124650659840, i64 %r1061)
  store i64 %call62, ptr %r19, align 4
  %r1963 = load i64, ptr %r19, align 4
  %box_str64 = or i64 ptrtoint (ptr @.str.30 to i64), -2533274790395904
  %box_str65 = or i64 ptrtoint (ptr @.str.39 to i64), -2533274790395904
  %call66 = call i64 @__bs_assertEqual(i64 -4222124650659840, i64 %r1963, i64 %box_str64, i64 %box_str65)
  store i64 %call66, ptr %r20, align 4
  %r167 = load i64, ptr %r1, align 4
  store i64 %r167, ptr %r21, align 4
  %r2168 = load i64, ptr %r21, align 4
  %prop_get = call i64 @__bs_prop_get(i64 %r2168, ptr @prop_str.38, i32 8)
  store i64 %prop_get, ptr %r23, align 4
  %r2369 = load i64, ptr %r23, align 4
  store i64 %r2369, ptr %r22, align 4
  %r2170 = load i64, ptr %r21, align 4
  %prop_get71 = call i64 @__bs_prop_get(i64 %r2170, ptr @prop_str.39, i32 11)
  store i64 %prop_get71, ptr %r25, align 4
  %r2572 = load i64, ptr %r25, align 4
  %eq_call = call i64 @__bs_strict_eq(i64 %r2572, i64 -4222124650659840)
  store i64 %eq_call, ptr %r26, align 4
  %r2673 = load i64, ptr %r26, align 4
  %t16 = lshr i64 %r2673, 48
  %is_tagged = icmp uge i64 %t16, 65521
  %tag_truthy = icmp uge i64 %t16, 65524
  %asf64 = bitcast i64 %r2673 to double
  %num_truthy = fcmp one double %asf64, 0.000000e+00
  %truthy = select i1 %is_tagged, i1 %tag_truthy, i1 %num_truthy
  br i1 %truthy, label %bb1, label %bb2

bb1:                                              ; preds = %bb0
  %call74 = call i64 @__bs_new_object()
  store i64 %call74, ptr %r28, align 4
  %r2875 = load i64, ptr %r28, align 4
  store i64 %r2875, ptr %r29, align 4
  %r2976 = load i64, ptr %r29, align 4
  store i64 %r2976, ptr %r27, align 4
  br label %bb3

bb2:                                              ; preds = %bb0
  %r2177 = load i64, ptr %r21, align 4
  %prop_get78 = call i64 @__bs_prop_get(i64 %r2177, ptr @prop_str.40, i32 11)
  store i64 %prop_get78, ptr %r30, align 4
  %r3079 = load i64, ptr %r30, align 4
  store i64 %r3079, ptr %r27, align 4
  br label %bb3

bb3:                                              ; preds = %bb2, %bb1
  %r2780 = load i64, ptr %r27, align 4
  %prop_get81 = call i64 @__bs_prop_get(i64 %r2780, ptr @prop_str.41, i32 8)
  store i64 %prop_get81, ptr %r31, align 4
  %r3182 = load i64, ptr %r31, align 4
  %eq_call83 = call i64 @__bs_strict_eq(i64 %r3182, i64 -4222124650659840)
  store i64 %eq_call83, ptr %r32, align 4
  %r3284 = load i64, ptr %r32, align 4
  %t1685 = lshr i64 %r3284, 48
  %is_tagged86 = icmp uge i64 %t1685, 65521
  %tag_truthy87 = icmp uge i64 %t1685, 65524
  %asf6488 = bitcast i64 %r3284 to double
  %num_truthy89 = fcmp one double %asf6488, 0.000000e+00
  %truthy90 = select i1 %is_tagged86, i1 %tag_truthy87, i1 %num_truthy89
  br i1 %truthy90, label %bb4, label %bb5

bb4:                                              ; preds = %bb3
  store i64 4624070917402656768, ptr %r33, align 4
  br label %bb6

bb5:                                              ; preds = %bb3
  %r2191 = load i64, ptr %r21, align 4
  %prop_get92 = call i64 @__bs_prop_get(i64 %r2191, ptr @prop_str.42, i32 11)
  store i64 %prop_get92, ptr %r34, align 4
  %r3493 = load i64, ptr %r34, align 4
  %eq_call94 = call i64 @__bs_strict_eq(i64 %r3493, i64 -4222124650659840)
  store i64 %eq_call94, ptr %r35, align 4
  %r3595 = load i64, ptr %r35, align 4
  %t1696 = lshr i64 %r3595, 48
  %is_tagged97 = icmp uge i64 %t1696, 65521
  %tag_truthy98 = icmp uge i64 %t1696, 65524
  %asf6499 = bitcast i64 %r3595 to double
  %num_truthy100 = fcmp one double %asf6499, 0.000000e+00
  %truthy101 = select i1 %is_tagged97, i1 %tag_truthy98, i1 %num_truthy100
  br i1 %truthy101, label %bb7, label %bb8

bb6:                                              ; preds = %bb9, %bb4
  %r33102 = load i64, ptr %r33, align 4
  store i64 %r33102, ptr %r24, align 4
  %r22103 = load i64, ptr %r22, align 4
  %box_str104 = or i64 ptrtoint (ptr @.str.31 to i64), -2533274790395904
  %box_str105 = or i64 ptrtoint (ptr @.str.40 to i64), -2533274790395904
  %call106 = call i64 @__bs_assertEqual(i64 -4222124650659840, i64 %r22103, i64 %box_str104, i64 %box_str105)
  store i64 %call106, ptr %r40, align 4
  %r24107 = load i64, ptr %r24, align 4
  %box_str108 = or i64 ptrtoint (ptr @.str.41 to i64), -2533274790395904
  %call109 = call i64 @__bs_assertEqual(i64 -4222124650659840, i64 %r24107, i64 4624070917402656768, i64 %box_str108)
  store i64 %call109, ptr %r41, align 4
  call void @__bs_shadow_pop()
  ret i64 -4222124650659840

bb7:                                              ; preds = %bb5
  %call110 = call i64 @__bs_new_object()
  store i64 %call110, ptr %r37, align 4
  %r37111 = load i64, ptr %r37, align 4
  store i64 %r37111, ptr %r29, align 4
  %r29112 = load i64, ptr %r29, align 4
  store i64 %r29112, ptr %r36, align 4
  br label %bb9

bb8:                                              ; preds = %bb5
  %r21113 = load i64, ptr %r21, align 4
  %prop_get114 = call i64 @__bs_prop_get(i64 %r21113, ptr @prop_str.43, i32 11)
  store i64 %prop_get114, ptr %r38, align 4
  %r38115 = load i64, ptr %r38, align 4
  store i64 %r38115, ptr %r36, align 4
  br label %bb9

bb9:                                              ; preds = %bb8, %bb7
  %r36116 = load i64, ptr %r36, align 4
  %prop_get117 = call i64 @__bs_prop_get(i64 %r36116, ptr @prop_str.44, i32 8)
  store i64 %prop_get117, ptr %r39, align 4
  %r39118 = load i64, ptr %r39, align 4
  store i64 %r39118, ptr %r33, align 4
  call void @__bs_safepoint_poll()
  br label %bb6
}

define i64 @__bs_formatCurrency(i64 %0, i64 %1, i64 %2) {
bb0:
  %regs_array = alloca [20 x i64], align 8
  %3 = ptrtoint ptr %regs_array to i64
  %regs_ptr = inttoptr i64 %3 to ptr
  %r0 = getelementptr i64, ptr %regs_array, i32 0
  store i64 -4222124650659840, ptr %r0, align 4
  %r1 = getelementptr i64, ptr %regs_array, i32 1
  store i64 -4222124650659840, ptr %r1, align 4
  %r2 = getelementptr i64, ptr %regs_array, i32 2
  store i64 -4222124650659840, ptr %r2, align 4
  %r3 = getelementptr i64, ptr %regs_array, i32 3
  store i64 -4222124650659840, ptr %r3, align 4
  %r4 = getelementptr i64, ptr %regs_array, i32 4
  store i64 -4222124650659840, ptr %r4, align 4
  %r5 = getelementptr i64, ptr %regs_array, i32 5
  store i64 -4222124650659840, ptr %r5, align 4
  %r6 = getelementptr i64, ptr %regs_array, i32 6
  store i64 -4222124650659840, ptr %r6, align 4
  %r7 = getelementptr i64, ptr %regs_array, i32 7
  store i64 -4222124650659840, ptr %r7, align 4
  %r8 = getelementptr i64, ptr %regs_array, i32 8
  store i64 -4222124650659840, ptr %r8, align 4
  %r9 = getelementptr i64, ptr %regs_array, i32 9
  store i64 -4222124650659840, ptr %r9, align 4
  %r10 = getelementptr i64, ptr %regs_array, i32 10
  store i64 -4222124650659840, ptr %r10, align 4
  %r11 = getelementptr i64, ptr %regs_array, i32 11
  store i64 -4222124650659840, ptr %r11, align 4
  %r12 = getelementptr i64, ptr %regs_array, i32 12
  store i64 -4222124650659840, ptr %r12, align 4
  %r13 = getelementptr i64, ptr %regs_array, i32 13
  store i64 -4222124650659840, ptr %r13, align 4
  %r14 = getelementptr i64, ptr %regs_array, i32 14
  store i64 -4222124650659840, ptr %r14, align 4
  %r15 = getelementptr i64, ptr %regs_array, i32 15
  store i64 -4222124650659840, ptr %r15, align 4
  %r16 = getelementptr i64, ptr %regs_array, i32 16
  store i64 -4222124650659840, ptr %r16, align 4
  %r17 = getelementptr i64, ptr %regs_array, i32 17
  store i64 -4222124650659840, ptr %r17, align 4
  %r18 = getelementptr i64, ptr %regs_array, i32 18
  store i64 -4222124650659840, ptr %r18, align 4
  %r19 = getelementptr i64, ptr %regs_array, i32 19
  store i64 -4222124650659840, ptr %r19, align 4
  %shadow_frame = alloca { ptr, i32, i32, ptr }, align 8
  %num_roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 1
  store i32 20, ptr %num_roots_ptr, align 4
  %roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 3
  store ptr %regs_ptr, ptr %roots_ptr, align 8
  call void @__bs_shadow_push(ptr %shadow_frame)
  store i64 %0, ptr %r0, align 4
  store i64 %1, ptr %r1, align 4
  store i64 %2, ptr %r2, align 4
  %r21 = load i64, ptr %r2, align 4
  store i64 %r21, ptr %r3, align 4
  %r110 = load i64, ptr %r1, align 4
  %call = call i64 @__bs_index_get(i64 %r110, i64 0)
  store i64 %call, ptr %r5, align 4
  %r511 = load i64, ptr %r5, align 4
  store i64 %r511, ptr %r4, align 4
  store i64 0, ptr %r6, align 4
  br label %bb1

bb1:                                              ; preds = %bb3, %bb0
  %r312 = load i64, ptr %r3, align 4
  %prop_get = call i64 @__bs_prop_get(i64 %r312, ptr @prop_str.45, i32 6)
  store i64 %prop_get, ptr %r7, align 4
  %r613 = load i64, ptr %r6, align 4
  %r714 = load i64, ptr %r7, align 4
  %unbox_num = bitcast i64 %r613 to double
  %unbox_num15 = bitcast i64 %r714 to double
  %cmp = fcmp olt double %unbox_num, %unbox_num15
  %box_bool = select i1 %cmp, i64 -3377699720527872, i64 -3659174697238528
  store i64 %box_bool, ptr %r8, align 4
  %r816 = load i64, ptr %r8, align 4
  %t16 = lshr i64 %r816, 48
  %is_tagged = icmp uge i64 %t16, 65521
  %tag_truthy = icmp uge i64 %t16, 65524
  %asf64 = bitcast i64 %r816 to double
  %num_truthy = fcmp one double %asf64, 0.000000e+00
  %truthy = select i1 %is_tagged, i1 %tag_truthy, i1 %num_truthy
  br i1 %truthy, label %bb2, label %bb4

bb2:                                              ; preds = %bb1
  %r317 = load i64, ptr %r3, align 4
  %r618 = load i64, ptr %r6, align 4
  %call19 = call i64 @__bs_index_get(i64 %r317, i64 %r618)
  store i64 %call19, ptr %r10, align 4
  %r1020 = load i64, ptr %r10, align 4
  store i64 %r1020, ptr %r9, align 4
  %r621 = load i64, ptr %r6, align 4
  %eq_call = call i64 @__bs_strict_eq(i64 %r621, i64 4611686018427387904)
  store i64 %eq_call, ptr %r12, align 4
  %r1222 = load i64, ptr %r12, align 4
  %t1623 = lshr i64 %r1222, 48
  %is_tagged24 = icmp uge i64 %t1623, 65521
  %tag_truthy25 = icmp uge i64 %t1623, 65524
  %asf6426 = bitcast i64 %r1222 to double
  %num_truthy27 = fcmp one double %asf6426, 0.000000e+00
  %truthy28 = select i1 %is_tagged24, i1 %tag_truthy25, i1 %num_truthy27
  br i1 %truthy28, label %bb5, label %bb6

bb3:                                              ; preds = %bb7
  %r629 = load i64, ptr %r6, align 4
  %add_call = call i64 @__bs_add(i64 %r629, i64 4607182418800017408)
  store i64 %add_call, ptr %r19, align 4
  %r1930 = load i64, ptr %r19, align 4
  store i64 %r1930, ptr %r6, align 4
  call void @__bs_safepoint_poll()
  br label %bb1

bb4:                                              ; preds = %bb1
  %r431 = load i64, ptr %r4, align 4
  call void @__bs_shadow_pop()
  ret i64 %r431

bb5:                                              ; preds = %bb2
  %box_str = or i64 ptrtoint (ptr @.str.42 to i64), -2533274790395904
  %r932 = load i64, ptr %r9, align 4
  %add_call33 = call i64 @__bs_add(i64 %box_str, i64 %r932)
  store i64 %add_call33, ptr %r14, align 4
  %r1434 = load i64, ptr %r14, align 4
  store i64 %r1434, ptr %r13, align 4
  br label %bb7

bb6:                                              ; preds = %bb2
  %r935 = load i64, ptr %r9, align 4
  store i64 %r935, ptr %r13, align 4
  br label %bb7

bb7:                                              ; preds = %bb6, %bb5
  %r1336 = load i64, ptr %r13, align 4
  store i64 %r1336, ptr %r11, align 4
  %r637 = load i64, ptr %r6, align 4
  %add_call38 = call i64 @__bs_add(i64 %r637, i64 4607182418800017408)
  store i64 %add_call38, ptr %r15, align 4
  %r139 = load i64, ptr %r1, align 4
  %r1540 = load i64, ptr %r15, align 4
  %call41 = call i64 @__bs_index_get(i64 %r139, i64 %r1540)
  store i64 %call41, ptr %r16, align 4
  %r1142 = load i64, ptr %r11, align 4
  %r1643 = load i64, ptr %r16, align 4
  %add_call44 = call i64 @__bs_add(i64 %r1142, i64 %r1643)
  store i64 %add_call44, ptr %r17, align 4
  %r445 = load i64, ptr %r4, align 4
  %r1746 = load i64, ptr %r17, align 4
  %add_call47 = call i64 @__bs_add(i64 %r445, i64 %r1746)
  store i64 %add_call47, ptr %r18, align 4
  %r1848 = load i64, ptr %r18, align 4
  store i64 %r1848, ptr %r4, align 4
  call void @__bs_safepoint_poll()
  br label %bb3
}

define i64 @__bs_runTemplateTests(i64 %0) {
bb0:
  %regs_array = alloca [17 x i64], align 8
  %1 = ptrtoint ptr %regs_array to i64
  %regs_ptr = inttoptr i64 %1 to ptr
  %r0 = getelementptr i64, ptr %regs_array, i32 0
  store i64 -4222124650659840, ptr %r0, align 4
  %r1 = getelementptr i64, ptr %regs_array, i32 1
  store i64 -4222124650659840, ptr %r1, align 4
  %r2 = getelementptr i64, ptr %regs_array, i32 2
  store i64 -4222124650659840, ptr %r2, align 4
  %r3 = getelementptr i64, ptr %regs_array, i32 3
  store i64 -4222124650659840, ptr %r3, align 4
  %r4 = getelementptr i64, ptr %regs_array, i32 4
  store i64 -4222124650659840, ptr %r4, align 4
  %r5 = getelementptr i64, ptr %regs_array, i32 5
  store i64 -4222124650659840, ptr %r5, align 4
  %r6 = getelementptr i64, ptr %regs_array, i32 6
  store i64 -4222124650659840, ptr %r6, align 4
  %r7 = getelementptr i64, ptr %regs_array, i32 7
  store i64 -4222124650659840, ptr %r7, align 4
  %r8 = getelementptr i64, ptr %regs_array, i32 8
  store i64 -4222124650659840, ptr %r8, align 4
  %r9 = getelementptr i64, ptr %regs_array, i32 9
  store i64 -4222124650659840, ptr %r9, align 4
  %r10 = getelementptr i64, ptr %regs_array, i32 10
  store i64 -4222124650659840, ptr %r10, align 4
  %r11 = getelementptr i64, ptr %regs_array, i32 11
  store i64 -4222124650659840, ptr %r11, align 4
  %r12 = getelementptr i64, ptr %regs_array, i32 12
  store i64 -4222124650659840, ptr %r12, align 4
  %r13 = getelementptr i64, ptr %regs_array, i32 13
  store i64 -4222124650659840, ptr %r13, align 4
  %r14 = getelementptr i64, ptr %regs_array, i32 14
  store i64 -4222124650659840, ptr %r14, align 4
  %r15 = getelementptr i64, ptr %regs_array, i32 15
  store i64 -4222124650659840, ptr %r15, align 4
  %r16 = getelementptr i64, ptr %regs_array, i32 16
  store i64 -4222124650659840, ptr %r16, align 4
  %shadow_frame = alloca { ptr, i32, i32, ptr }, align 8
  %num_roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 1
  store i32 17, ptr %num_roots_ptr, align 4
  %roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 3
  store ptr %regs_ptr, ptr %roots_ptr, align 8
  call void @__bs_shadow_push(ptr %shadow_frame)
  store i64 %0, ptr %r0, align 4
  %r01 = load i64, ptr %r0, align 4
  %payload = and i64 %r01, 281474976710655
  %obj_ptr = inttoptr i64 %payload to ptr
  %field_ptr = getelementptr i64, ptr %obj_ptr, i32 2
  %loaded = load i64, ptr %field_ptr, align 4
  store i64 %loaded, ptr %r1, align 4
  %box_str = or i64 ptrtoint (ptr @.str.43 to i64), -2533274790395904
  store i64 %box_str, ptr %r2, align 4
  store i64 4618430158869375222, ptr %r3, align 4
  store i64 4613937818241073152, ptr %r4, align 4
  %call = call i64 @__bs_array_new()
  store i64 %call, ptr %r6, align 4
  %r62 = load i64, ptr %r6, align 4
  %box_str3 = or i64 ptrtoint (ptr @.str.44 to i64), -2533274790395904
  %call4 = call i64 @__bs_array_push(i64 %r62, i64 %box_str3)
  store i64 %call4, ptr %r7, align 4
  %r65 = load i64, ptr %r6, align 4
  %box_str6 = or i64 ptrtoint (ptr @.str.45 to i64), -2533274790395904
  %call7 = call i64 @__bs_array_push(i64 %r65, i64 %box_str6)
  store i64 %call7, ptr %r8, align 4
  %r68 = load i64, ptr %r6, align 4
  %box_str9 = or i64 ptrtoint (ptr @.str.46 to i64), -2533274790395904
  %call10 = call i64 @__bs_array_push(i64 %r68, i64 %box_str9)
  store i64 %call10, ptr %r9, align 4
  %r611 = load i64, ptr %r6, align 4
  %box_str12 = or i64 ptrtoint (ptr @.str.47 to i64), -2533274790395904
  %call13 = call i64 @__bs_array_push(i64 %r611, i64 %box_str12)
  store i64 %call13, ptr %r10, align 4
  %call14 = call i64 @__bs_array_new()
  store i64 %call14, ptr %r11, align 4
  %r1115 = load i64, ptr %r11, align 4
  %r416 = load i64, ptr %r4, align 4
  %call17 = call i64 @__bs_array_push(i64 %r1115, i64 %r416)
  store i64 %call17, ptr %r12, align 4
  %r1118 = load i64, ptr %r11, align 4
  %r219 = load i64, ptr %r2, align 4
  %call20 = call i64 @__bs_array_push(i64 %r1118, i64 %r219)
  store i64 %call20, ptr %r13, align 4
  %r1121 = load i64, ptr %r11, align 4
  %r322 = load i64, ptr %r3, align 4
  %call23 = call i64 @__bs_array_push(i64 %r1121, i64 %r322)
  store i64 %call23, ptr %r14, align 4
  %r124 = load i64, ptr %r1, align 4
  %payload25 = and i64 %r124, 281474976710655
  %closure_ptr = inttoptr i64 %payload25 to ptr
  %fn_slot = getelementptr ptr, ptr %closure_ptr, i32 0
  %fn_ptr = load ptr, ptr %fn_slot, align 8
  %r126 = load i64, ptr %r1, align 4
  %r627 = load i64, ptr %r6, align 4
  %r1128 = load i64, ptr %r11, align 4
  %closure_call = call i64 %fn_ptr(i64 %r126, i64 %r627, i64 %r1128)
  store i64 %closure_call, ptr %r15, align 4
  %r1529 = load i64, ptr %r15, align 4
  store i64 %r1529, ptr %r5, align 4
  %r530 = load i64, ptr %r5, align 4
  %box_str31 = or i64 ptrtoint (ptr @.str.48 to i64), -2533274790395904
  %box_str32 = or i64 ptrtoint (ptr @.str.49 to i64), -2533274790395904
  %call33 = call i64 @__bs_assertEqual(i64 -4222124650659840, i64 %r530, i64 %box_str31, i64 %box_str32)
  store i64 %call33, ptr %r16, align 4
  call void @__bs_shadow_pop()
  ret i64 -4222124650659840
}

define i64 @__bs_main(i64 %0) {
bb0:
  %regs_array = alloca [8 x i64], align 8
  %1 = ptrtoint ptr %regs_array to i64
  %regs_ptr = inttoptr i64 %1 to ptr
  %r0 = getelementptr i64, ptr %regs_array, i32 0
  store i64 -4222124650659840, ptr %r0, align 4
  %r1 = getelementptr i64, ptr %regs_array, i32 1
  store i64 -4222124650659840, ptr %r1, align 4
  %r2 = getelementptr i64, ptr %regs_array, i32 2
  store i64 -4222124650659840, ptr %r2, align 4
  %r3 = getelementptr i64, ptr %regs_array, i32 3
  store i64 -4222124650659840, ptr %r3, align 4
  %r4 = getelementptr i64, ptr %regs_array, i32 4
  store i64 -4222124650659840, ptr %r4, align 4
  %r5 = getelementptr i64, ptr %regs_array, i32 5
  store i64 -4222124650659840, ptr %r5, align 4
  %r6 = getelementptr i64, ptr %regs_array, i32 6
  store i64 -4222124650659840, ptr %r6, align 4
  %r7 = getelementptr i64, ptr %regs_array, i32 7
  store i64 -4222124650659840, ptr %r7, align 4
  %shadow_frame = alloca { ptr, i32, i32, ptr }, align 8
  %num_roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 1
  store i32 8, ptr %num_roots_ptr, align 4
  %roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 3
  store ptr %regs_ptr, ptr %roots_ptr, align 8
  call void @__bs_shadow_push(ptr %shadow_frame)
  store i64 %0, ptr %r0, align 4
  %box_str = or i64 ptrtoint (ptr @.str.50 to i64), -2533274790395904
  call void @__bs_console_log_1(i64 %box_str)
  store i64 -4222124650659840, ptr %r1, align 4
  %call = call i64 @__bs_runCartTests(i64 -4222124650659840)
  store i64 %call, ptr %r2, align 4
  %box_str1 = or i64 ptrtoint (ptr @.str.51 to i64), -2533274790395904
  call void @__bs_console_log_1(i64 %box_str1)
  store i64 -4222124650659840, ptr %r3, align 4
  %call2 = call i64 @__bs_runProfileTests(i64 -4222124650659840)
  store i64 %call2, ptr %r4, align 4
  %box_str3 = or i64 ptrtoint (ptr @.str.52 to i64), -2533274790395904
  call void @__bs_console_log_1(i64 %box_str3)
  store i64 -4222124650659840, ptr %r5, align 4
  %call4 = call i64 @__bs_runTemplateTests(i64 -4222124650659840)
  store i64 %call4, ptr %r6, align 4
  %box_str5 = or i64 ptrtoint (ptr @.str.53 to i64), -2533274790395904
  call void @__bs_console_log_1(i64 %box_str5)
  store i64 -4222124650659840, ptr %r7, align 4
  call void @__bs_shadow_pop()
  ret i64 -4222124650659840
}

define i32 @main() {
bb0:
  %r0 = alloca i64, align 8
  store i64 -4222124650659840, ptr %r0, align 4
  %r1 = alloca i64, align 8
  store i64 -4222124650659840, ptr %r1, align 4
  %r2 = alloca i64, align 8
  store i64 -4222124650659840, ptr %r2, align 4
  %r3 = alloca i64, align 8
  store i64 -4222124650659840, ptr %r3, align 4
  %r4 = alloca i64, align 8
  store i64 -4222124650659840, ptr %r4, align 4
  %r5 = alloca i64, align 8
  store i64 -4222124650659840, ptr %r5, align 4
  %r6 = alloca i64, align 8
  store i64 -4222124650659840, ptr %r6, align 4
  %r7 = alloca i64, align 8
  store i64 -4222124650659840, ptr %r7, align 4
  %r8 = alloca i64, align 8
  store i64 -4222124650659840, ptr %r8, align 4
  %r9 = alloca i64, align 8
  store i64 -4222124650659840, ptr %r9, align 4
  %r10 = alloca i64, align 8
  store i64 -4222124650659840, ptr %r10, align 4
  %r11 = alloca i64, align 8
  store i64 -4222124650659840, ptr %r11, align 4
  %r12 = alloca i64, align 8
  store i64 -4222124650659840, ptr %r12, align 4
  %r13 = alloca i64, align 8
  store i64 -4222124650659840, ptr %r13, align 4
  %r14 = alloca i64, align 8
  store i64 -4222124650659840, ptr %r14, align 4
  %r15 = alloca i64, align 8
  store i64 -4222124650659840, ptr %r15, align 4
  %r16 = alloca i64, align 8
  store i64 -4222124650659840, ptr %r16, align 4
  %r17 = alloca i64, align 8
  store i64 -4222124650659840, ptr %r17, align 4
  %r18 = alloca i64, align 8
  store i64 -4222124650659840, ptr %r18, align 4
  %alloc_closure = call i64 @__bs_alloc_closure(i64 16)
  %payload = and i64 %alloc_closure, 281474976710655
  %closure_ptr = inttoptr i64 %payload to ptr
  %fn_slot = getelementptr ptr, ptr %closure_ptr, i32 0
  store ptr @__bs_assertEqual, ptr %fn_slot, align 8
  %unused_slot = getelementptr i64, ptr %closure_ptr, i32 1
  store i64 -4222124650659840, ptr %unused_slot, align 4
  store i64 %alloc_closure, ptr %r1, align 4
  %r19 = load i64, ptr %r1, align 4
  store i64 %r19, ptr %r0, align 4
  %alloc_closure10 = call i64 @__bs_alloc_closure(i64 16)
  %payload11 = and i64 %alloc_closure10, 281474976710655
  %closure_ptr12 = inttoptr i64 %payload11 to ptr
  %fn_slot13 = getelementptr ptr, ptr %closure_ptr12, i32 0
  store ptr @__bs_calculateCartTotal, ptr %fn_slot13, align 8
  %unused_slot14 = getelementptr i64, ptr %closure_ptr12, i32 1
  store i64 -4222124650659840, ptr %unused_slot14, align 4
  store i64 %alloc_closure10, ptr %r3, align 4
  %r315 = load i64, ptr %r3, align 4
  store i64 %r315, ptr %r2, align 4
  %alloc_closure16 = call i64 @__bs_alloc_closure(i64 16)
  %payload17 = and i64 %alloc_closure16, 281474976710655
  %closure_ptr18 = inttoptr i64 %payload17 to ptr
  %fn_slot19 = getelementptr ptr, ptr %closure_ptr18, i32 0
  store ptr @__bs_runCartTests, ptr %fn_slot19, align 8
  %unused_slot20 = getelementptr i64, ptr %closure_ptr18, i32 1
  store i64 -4222124650659840, ptr %unused_slot20, align 4
  store i64 %alloc_closure16, ptr %r5, align 4
  %r521 = load i64, ptr %r5, align 4
  store i64 %r521, ptr %r4, align 4
  %alloc_closure22 = call i64 @__bs_alloc_closure(i64 16)
  %payload23 = and i64 %alloc_closure22, 281474976710655
  %closure_ptr24 = inttoptr i64 %payload23 to ptr
  %fn_slot25 = getelementptr ptr, ptr %closure_ptr24, i32 0
  store ptr @__bs_getUserTheme, ptr %fn_slot25, align 8
  %unused_slot26 = getelementptr i64, ptr %closure_ptr24, i32 1
  store i64 -4222124650659840, ptr %unused_slot26, align 4
  store i64 %alloc_closure22, ptr %r7, align 4
  %r727 = load i64, ptr %r7, align 4
  store i64 %r727, ptr %r6, align 4
  %alloc_closure28 = call i64 @__bs_alloc_closure(i64 16)
  %payload29 = and i64 %alloc_closure28, 281474976710655
  %closure_ptr30 = inttoptr i64 %payload29 to ptr
  %fn_slot31 = getelementptr ptr, ptr %closure_ptr30, i32 0
  store ptr @__bs_getUserCity, ptr %fn_slot31, align 8
  %unused_slot32 = getelementptr i64, ptr %closure_ptr30, i32 1
  store i64 -4222124650659840, ptr %unused_slot32, align 4
  store i64 %alloc_closure28, ptr %r9, align 4
  %r933 = load i64, ptr %r9, align 4
  store i64 %r933, ptr %r8, align 4
  %alloc_closure34 = call i64 @__bs_alloc_closure(i64 16)
  %payload35 = and i64 %alloc_closure34, 281474976710655
  %closure_ptr36 = inttoptr i64 %payload35 to ptr
  %fn_slot37 = getelementptr ptr, ptr %closure_ptr36, i32 0
  store ptr @__bs_runProfileTests, ptr %fn_slot37, align 8
  %unused_slot38 = getelementptr i64, ptr %closure_ptr36, i32 1
  store i64 -4222124650659840, ptr %unused_slot38, align 4
  store i64 %alloc_closure34, ptr %r11, align 4
  %r1139 = load i64, ptr %r11, align 4
  store i64 %r1139, ptr %r10, align 4
  %alloc_closure40 = call i64 @__bs_alloc_closure(i64 16)
  %payload41 = and i64 %alloc_closure40, 281474976710655
  %closure_ptr42 = inttoptr i64 %payload41 to ptr
  %fn_slot43 = getelementptr ptr, ptr %closure_ptr42, i32 0
  store ptr @__bs_formatCurrency, ptr %fn_slot43, align 8
  %unused_slot44 = getelementptr i64, ptr %closure_ptr42, i32 1
  store i64 -4222124650659840, ptr %unused_slot44, align 4
  store i64 %alloc_closure40, ptr %r13, align 4
  %r1345 = load i64, ptr %r13, align 4
  store i64 %r1345, ptr %r12, align 4
  %alloc_closure46 = call i64 @__bs_alloc_closure(i64 24)
  %payload47 = and i64 %alloc_closure46, 281474976710655
  %closure_ptr48 = inttoptr i64 %payload47 to ptr
  %fn_slot49 = getelementptr ptr, ptr %closure_ptr48, i32 0
  store ptr @__bs_runTemplateTests, ptr %fn_slot49, align 8
  %unused_slot50 = getelementptr i64, ptr %closure_ptr48, i32 1
  store i64 -4222124650659840, ptr %unused_slot50, align 4
  %r1251 = load i64, ptr %r12, align 4
  %capture_slot = getelementptr i64, ptr %closure_ptr48, i32 2
  store i64 %r1251, ptr %capture_slot, align 4
  store i64 %alloc_closure46, ptr %r15, align 4
  %r1552 = load i64, ptr %r15, align 4
  store i64 %r1552, ptr %r14, align 4
  %alloc_closure53 = call i64 @__bs_alloc_closure(i64 16)
  %payload54 = and i64 %alloc_closure53, 281474976710655
  %closure_ptr55 = inttoptr i64 %payload54 to ptr
  %fn_slot56 = getelementptr ptr, ptr %closure_ptr55, i32 0
  store ptr @__bs_main, ptr %fn_slot56, align 8
  %unused_slot57 = getelementptr i64, ptr %closure_ptr55, i32 1
  store i64 -4222124650659840, ptr %unused_slot57, align 4
  store i64 %alloc_closure53, ptr %r17, align 4
  %r1758 = load i64, ptr %r17, align 4
  store i64 %r1758, ptr %r16, align 4
  %call = call i64 @__bs_main(i64 -4222124650659840)
  store i64 %call, ptr %r18, align 4
  call void @__bs_drain_microtasks()
  ret i32 0
}

attributes #0 = { returns_twice }

