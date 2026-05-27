; ModuleID = 'tests/hello.ts'
source_filename = "tests/hello.ts"

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
@prop_str = private unnamed_addr constant [5 x i8] c"name\00", align 1
@.str.11 = unnamed_addr constant [6 x i8] c"samon\00"
@prop_str.2 = private unnamed_addr constant [4 x i8] c"age\00", align 1
@prop_str.3 = private unnamed_addr constant [4 x i8] c"age\00", align 1
@prop_str.4 = private unnamed_addr constant [5 x i8] c"name\00", align 1
@.str.12 = unnamed_addr constant [7 x i8] c"hello,\00"
@.str.13 = unnamed_addr constant [2 x i8] c",\00"
@prop_str.5 = private unnamed_addr constant [4 x i8] c"age\00", align 1
@.str.14 = unnamed_addr constant [17 x i8] c" , your id is : \00"

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

declare i64 @__bs_get_globalThis()

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

declare i64 @__bs_Number.1(i64)

declare i64 @__bs_in(i64, i64)

declare i64 @__bs_delete_prop(i64, i64)

declare i64 @__bs_call_push(i64, i64, i64)

declare i64 @__bs_call_pop(i64, i64)

declare i64 @__bs_call_slice(i64, i64, i64, i64)

declare i64 @__bs_call_indexOf(i64, i64, i64)

declare i64 @__bs_call_includes(i64, i64, i64)

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

done:                                             ; preds = %print_closure, %print_undef, %print_null, %default_name, %load_name, %print_str, %print_false, %print_true, %print_num
  %8 = call i32 @putchar(i32 10)
  ret void

load_name:                                        ; preds = %print_obj
  %name_ptr_ptr = getelementptr ptr, ptr %vtable_ptr, i32 1
  %name_ptr = load ptr, ptr %name_ptr_ptr, align 8
  %9 = call i32 (ptr, ...) @printf(ptr @.str.4, ptr %name_ptr)
  br label %done

default_name:                                     ; preds = %print_obj
  %10 = call i32 (ptr, ...) @printf(ptr @.str.4, ptr @.str.5)
  br label %done
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
  %call = call i64 @__bs_new_object()
  store i64 %call, ptr %r1, align 4
  %r16 = load i64, ptr %r1, align 4
  store i64 %r16, ptr %r2, align 4
  %r27 = load i64, ptr %r2, align 4
  %box_str = or i64 ptrtoint (ptr @.str.11 to i64), -2533274790395904
  %prop_set = call i64 @__bs_prop_set(i64 %r27, ptr @prop_str, i32 4, i64 %box_str)
  %r28 = load i64, ptr %r2, align 4
  %prop_set9 = call i64 @__bs_prop_set(i64 %r28, ptr @prop_str.2, i32 3, i64 4627730092099895296)
  %r210 = load i64, ptr %r2, align 4
  store i64 %r210, ptr %r0, align 4
  store i64 4617315517961601024, ptr %r3, align 4
  %r311 = load i64, ptr %r3, align 4
  %add_call = call i64 @__bs_add(i64 %r311, i64 4617315517961601024)
  store i64 %add_call, ptr %r4, align 4
  %r012 = load i64, ptr %r0, align 4
  %prop_get = call i64 @__bs_prop_get(i64 %r012, ptr @prop_str.3, i32 3)
  store i64 %prop_get, ptr %r5, align 4
  %r413 = load i64, ptr %r4, align 4
  %r514 = load i64, ptr %r5, align 4
  %unbox_num = bitcast i64 %r413 to double
  %unbox_num15 = bitcast i64 %r514 to double
  %mul = fmul double %unbox_num, %unbox_num15
  %box_num = bitcast double %mul to i64
  store i64 %box_num, ptr %r6, align 4
  %r316 = load i64, ptr %r3, align 4
  %r617 = load i64, ptr %r6, align 4
  %unbox_num18 = bitcast i64 %r316 to double
  %unbox_num19 = bitcast i64 %r617 to double
  %mul20 = fmul double %unbox_num18, %unbox_num19
  %box_num21 = bitcast double %mul20 to i64
  store i64 %box_num21, ptr %r7, align 4
  %r722 = load i64, ptr %r7, align 4
  store i64 %r722, ptr %r3, align 4
  %r023 = load i64, ptr %r0, align 4
  %prop_get24 = call i64 @__bs_prop_get(i64 %r023, ptr @prop_str.4, i32 4)
  store i64 %prop_get24, ptr %r8, align 4
  %box_str25 = or i64 ptrtoint (ptr @.str.12 to i64), -2533274790395904
  %r826 = load i64, ptr %r8, align 4
  %add_call27 = call i64 @__bs_add(i64 %box_str25, i64 %r826)
  store i64 %add_call27, ptr %r9, align 4
  %r928 = load i64, ptr %r9, align 4
  %box_str29 = or i64 ptrtoint (ptr @.str.13 to i64), -2533274790395904
  %add_call30 = call i64 @__bs_add(i64 %r928, i64 %box_str29)
  store i64 %add_call30, ptr %r10, align 4
  %r031 = load i64, ptr %r0, align 4
  %prop_get32 = call i64 @__bs_prop_get(i64 %r031, ptr @prop_str.5, i32 3)
  store i64 %prop_get32, ptr %r11, align 4
  %r1033 = load i64, ptr %r10, align 4
  %r1134 = load i64, ptr %r11, align 4
  %add_call35 = call i64 @__bs_add(i64 %r1033, i64 %r1134)
  store i64 %add_call35, ptr %r12, align 4
  %r1236 = load i64, ptr %r12, align 4
  %box_str37 = or i64 ptrtoint (ptr @.str.14 to i64), -2533274790395904
  %add_call38 = call i64 @__bs_add(i64 %r1236, i64 %box_str37)
  store i64 %add_call38, ptr %r13, align 4
  %r1339 = load i64, ptr %r13, align 4
  %r340 = load i64, ptr %r3, align 4
  %add_call41 = call i64 @__bs_add(i64 %r1339, i64 %r340)
  store i64 %add_call41, ptr %r14, align 4
  %r1442 = load i64, ptr %r14, align 4
  call void @__bs_console_log_1(i64 %r1442)
  store i64 -4222124650659840, ptr %r15, align 4
  call void @__bs_drain_microtasks()
  ret i32 0
}

attributes #0 = { returns_twice }

