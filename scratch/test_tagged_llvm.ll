; ModuleID = 'scratch/test_tagged.ts'
source_filename = "scratch/test_tagged.ts"

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
@.str.14 = unnamed_addr constant [5 x i8] c" -> \00"
@.str.15 = unnamed_addr constant [5 x i8] c"PASS\00"
@.str.16 = unnamed_addr constant [5 x i8] c"FAIL\00"
@prop_str = private unnamed_addr constant [7 x i8] c"length\00", align 1
@.str.17 = unnamed_addr constant [2 x i8] c"$\00"
@.str.18 = unnamed_addr constant [15 x i8] c"Premium Coffee\00"
@.str.19 = unnamed_addr constant [10 x i8] c"Receipt: \00"
@.str.20 = unnamed_addr constant [3 x i8] c"x \00"
@.str.21 = unnamed_addr constant [5 x i8] c" at \00"
@.str.22 = unnamed_addr constant [6 x i8] c" each\00"
@.str.23 = unnamed_addr constant [41 x i8] c"Receipt: 3x Premium Coffee at $5.99 each\00"
@.str.24 = unnamed_addr constant [32 x i8] c"Invoice currency formatting tag\00"

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
  %regs_array = alloca [15 x i64], align 8
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
  %shadow_frame = alloca { ptr, i32, i32, ptr }, align 8
  %num_roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 1
  store i32 15, ptr %num_roots_ptr, align 4
  %roots_ptr = getelementptr inbounds nuw { ptr, i32, i32, ptr }, ptr %shadow_frame, i32 0, i32 3
  store ptr %regs_ptr, ptr %roots_ptr, align 8
  call void @__bs_shadow_push(ptr %shadow_frame)
  store i64 %0, ptr %r0, align 4
  store i64 %1, ptr %r1, align 4
  store i64 %2, ptr %r2, align 4
  store i64 %3, ptr %r3, align 4
  %r15 = load i64, ptr %r1, align 4
  %r26 = load i64, ptr %r2, align 4
  %eq_call = call i64 @__bs_strict_eq(i64 %r15, i64 %r26)
  store i64 %eq_call, ptr %r5, align 4
  %r57 = load i64, ptr %r5, align 4
  store i64 %r57, ptr %r4, align 4
  %box_str = or i64 ptrtoint (ptr @.str.11 to i64), -2533274790395904
  %r38 = load i64, ptr %r3, align 4
  %add_call = call i64 @__bs_add(i64 %box_str, i64 %r38)
  store i64 %add_call, ptr %r6, align 4
  %r69 = load i64, ptr %r6, align 4
  %box_str10 = or i64 ptrtoint (ptr @.str.12 to i64), -2533274790395904
  %add_call11 = call i64 @__bs_add(i64 %r69, i64 %box_str10)
  store i64 %add_call11, ptr %r7, align 4
  %r712 = load i64, ptr %r7, align 4
  %r213 = load i64, ptr %r2, align 4
  %add_call14 = call i64 @__bs_add(i64 %r712, i64 %r213)
  store i64 %add_call14, ptr %r8, align 4
  %r815 = load i64, ptr %r8, align 4
  %box_str16 = or i64 ptrtoint (ptr @.str.13 to i64), -2533274790395904
  %add_call17 = call i64 @__bs_add(i64 %r815, i64 %box_str16)
  store i64 %add_call17, ptr %r9, align 4
  %r918 = load i64, ptr %r9, align 4
  %r119 = load i64, ptr %r1, align 4
  %add_call20 = call i64 @__bs_add(i64 %r918, i64 %r119)
  store i64 %add_call20, ptr %r10, align 4
  %r1021 = load i64, ptr %r10, align 4
  %box_str22 = or i64 ptrtoint (ptr @.str.14 to i64), -2533274790395904
  %add_call23 = call i64 @__bs_add(i64 %r1021, i64 %box_str22)
  store i64 %add_call23, ptr %r11, align 4
  %r424 = load i64, ptr %r4, align 4
  %t16 = lshr i64 %r424, 48
  %is_tagged = icmp uge i64 %t16, 65521
  %tag_truthy = icmp uge i64 %t16, 65524
  %asf64 = bitcast i64 %r424 to double
  %num_truthy = fcmp one double %asf64, 0.000000e+00
  %truthy = select i1 %is_tagged, i1 %tag_truthy, i1 %num_truthy
  br i1 %truthy, label %bb1, label %bb2

bb1:                                              ; preds = %bb0
  %box_str25 = or i64 ptrtoint (ptr @.str.15 to i64), -2533274790395904
  store i64 %box_str25, ptr %r12, align 4
  br label %bb3

bb2:                                              ; preds = %bb0
  %box_str26 = or i64 ptrtoint (ptr @.str.16 to i64), -2533274790395904
  store i64 %box_str26, ptr %r12, align 4
  br label %bb3

bb3:                                              ; preds = %bb2, %bb1
  %r1127 = load i64, ptr %r11, align 4
  %r1228 = load i64, ptr %r12, align 4
  %add_call29 = call i64 @__bs_add(i64 %r1127, i64 %r1228)
  store i64 %add_call29, ptr %r13, align 4
  %r1330 = load i64, ptr %r13, align 4
  call void @__bs_console_log_1(i64 %r1330)
  store i64 -4222124650659840, ptr %r14, align 4
  ret i64 -4222124650659840
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
  %prop_get = call i64 @__bs_prop_get(i64 %r312, ptr @prop_str, i32 6)
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
  ret i64 %r431

bb5:                                              ; preds = %bb2
  %box_str = or i64 ptrtoint (ptr @.str.17 to i64), -2533274790395904
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

define i64 @__bs_main(i64 %0) {
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
  %box_str = or i64 ptrtoint (ptr @.str.18 to i64), -2533274790395904
  store i64 %box_str, ptr %r2, align 4
  store i64 4618430158869375222, ptr %r3, align 4
  store i64 4613937818241073152, ptr %r4, align 4
  %call = call i64 @__bs_array_new()
  store i64 %call, ptr %r6, align 4
  %r62 = load i64, ptr %r6, align 4
  %box_str3 = or i64 ptrtoint (ptr @.str.19 to i64), -2533274790395904
  %call4 = call i64 @__bs_array_push(i64 %r62, i64 %box_str3)
  store i64 %call4, ptr %r7, align 4
  %r65 = load i64, ptr %r6, align 4
  %box_str6 = or i64 ptrtoint (ptr @.str.20 to i64), -2533274790395904
  %call7 = call i64 @__bs_array_push(i64 %r65, i64 %box_str6)
  store i64 %call7, ptr %r8, align 4
  %r68 = load i64, ptr %r6, align 4
  %box_str9 = or i64 ptrtoint (ptr @.str.21 to i64), -2533274790395904
  %call10 = call i64 @__bs_array_push(i64 %r68, i64 %box_str9)
  store i64 %call10, ptr %r9, align 4
  %r611 = load i64, ptr %r6, align 4
  %box_str12 = or i64 ptrtoint (ptr @.str.22 to i64), -2533274790395904
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
  %box_str31 = or i64 ptrtoint (ptr @.str.23 to i64), -2533274790395904
  %box_str32 = or i64 ptrtoint (ptr @.str.24 to i64), -2533274790395904
  %call33 = call i64 @__bs_assertEqual(i64 -4222124650659840, i64 %r530, i64 %box_str31, i64 %box_str32)
  store i64 %call33, ptr %r16, align 4
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
  %alloc_closure = call i64 @__bs_alloc_closure(i64 16)
  %payload = and i64 %alloc_closure, 281474976710655
  %closure_ptr = inttoptr i64 %payload to ptr
  %fn_slot = getelementptr ptr, ptr %closure_ptr, i32 0
  store ptr @__bs_assertEqual, ptr %fn_slot, align 8
  %unused_slot = getelementptr i64, ptr %closure_ptr, i32 1
  store i64 -4222124650659840, ptr %unused_slot, align 4
  store i64 %alloc_closure, ptr %r1, align 4
  %r11 = load i64, ptr %r1, align 4
  store i64 %r11, ptr %r0, align 4
  %alloc_closure2 = call i64 @__bs_alloc_closure(i64 16)
  %payload3 = and i64 %alloc_closure2, 281474976710655
  %closure_ptr4 = inttoptr i64 %payload3 to ptr
  %fn_slot5 = getelementptr ptr, ptr %closure_ptr4, i32 0
  store ptr @__bs_formatCurrency, ptr %fn_slot5, align 8
  %unused_slot6 = getelementptr i64, ptr %closure_ptr4, i32 1
  store i64 -4222124650659840, ptr %unused_slot6, align 4
  store i64 %alloc_closure2, ptr %r3, align 4
  %r37 = load i64, ptr %r3, align 4
  store i64 %r37, ptr %r2, align 4
  %alloc_closure8 = call i64 @__bs_alloc_closure(i64 24)
  %payload9 = and i64 %alloc_closure8, 281474976710655
  %closure_ptr10 = inttoptr i64 %payload9 to ptr
  %fn_slot11 = getelementptr ptr, ptr %closure_ptr10, i32 0
  store ptr @__bs_main, ptr %fn_slot11, align 8
  %unused_slot12 = getelementptr i64, ptr %closure_ptr10, i32 1
  store i64 -4222124650659840, ptr %unused_slot12, align 4
  %r213 = load i64, ptr %r2, align 4
  %capture_slot = getelementptr i64, ptr %closure_ptr10, i32 2
  store i64 %r213, ptr %capture_slot, align 4
  store i64 %alloc_closure8, ptr %r5, align 4
  %r514 = load i64, ptr %r5, align 4
  store i64 %r514, ptr %r4, align 4
  %r415 = load i64, ptr %r4, align 4
  %payload16 = and i64 %r415, 281474976710655
  %closure_ptr17 = inttoptr i64 %payload16 to ptr
  %fn_slot18 = getelementptr ptr, ptr %closure_ptr17, i32 0
  %fn_ptr = load ptr, ptr %fn_slot18, align 8
  %r419 = load i64, ptr %r4, align 4
  %closure_call = call i64 %fn_ptr(i64 %r419)
  store i64 %closure_call, ptr %r6, align 4
  call void @__bs_drain_microtasks()
  ret i32 0
}

attributes #0 = { returns_twice }

