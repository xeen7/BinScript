use crate::dynamic_call::helpers::{TAG_MASK, TAG_ARRAY, TAG_STRING, TAG_OBJECT, PAYLOAD_MASK};


// Wrapper for forEach because it returns void
pub unsafe extern "C-unwind" fn array_for_each_wrapper(arr: u64, cb: u64) -> u64 {
    crate::array::__bs_array_forEach(arr, cb);
    0
}

// Non-existent string dummy fallback functions for methods only present on arrays
pub unsafe extern "C-unwind" fn dummy_str_0(_: u64) -> u64 { crate::circ::box_number(0.0) }
pub unsafe extern "C-unwind" fn dummy_str_1(_: u64, _: u64) -> u64 { crate::circ::box_number(0.0) }
pub unsafe extern "C-unwind" fn dummy_str_2(_: u64, _: u64, _: u64) -> u64 { crate::circ::box_number(0.0) }
pub unsafe extern "C-unwind" fn dummy_str_3(_: u64, _: u64, _: u64, _: u64) -> u64 { crate::circ::box_number(0.0) }

// Non-existent array dummy fallback functions for methods only present on strings
pub unsafe extern "C-unwind" fn dummy_arr_0(_: u64) -> u64 { 0 }
pub unsafe extern "C-unwind" fn dummy_arr_1(_: u64, _: u64) -> u64 { 0 }
pub unsafe extern "C-unwind" fn dummy_arr_2(_: u64, _: u64, _: u64) -> u64 { 0 }

// Array & String Dispatchers
crate::dispatch_1_arg!(__bs_call_push, "push", crate::array::__bs_array_push, dummy_str_1);
crate::dispatch_0_args!(__bs_call_pop, "pop", crate::array::__bs_array_pop, dummy_str_0);
crate::dispatch_2_args!(__bs_call_slice, "slice", crate::array::__bs_array_slice, crate::string::__bs_string_substring);
crate::dispatch_1_arg!(__bs_call_includes, "includes", crate::array::__bs_array_includes, crate::string::__bs_string_includes);
crate::dispatch_1_arg!(__bs_call_join, "join", crate::array::__bs_array_join, dummy_str_1);
crate::dispatch_0_args!(__bs_call_reverse, "reverse", crate::array::__bs_array_reverse, dummy_str_0);
crate::dispatch_1_arg!(__bs_call_concat, "concat", crate::array::__bs_array_concat, dummy_str_1);
crate::dispatch_3_args!(__bs_call_fill, "fill", crate::array::__bs_array_fill, dummy_str_3);

crate::dispatch_1_arg!(__bs_call_forEach, "forEach", array_for_each_wrapper, dummy_str_1);
crate::dispatch_1_arg!(__bs_call_map, "map", crate::array::__bs_array_map, dummy_str_1);
crate::dispatch_1_arg!(__bs_call_filter, "filter", crate::array::__bs_array_filter, dummy_str_1);
crate::dispatch_1_arg!(__bs_call_find, "find", crate::array::__bs_array_find, dummy_str_1);
crate::dispatch_1_arg!(__bs_call_findIndex, "findIndex", crate::array::__bs_array_findIndex, dummy_str_1);
crate::dispatch_1_arg!(__bs_call_every, "every", crate::array::__bs_array_every, dummy_str_1);
crate::dispatch_1_arg!(__bs_call_some, "some", crate::array::__bs_array_some, dummy_str_1);
crate::dispatch_2_args!(__bs_call_reduce, "reduce", crate::array::__bs_array_reduce, dummy_str_2);

// String-only Dispatchers
crate::dispatch_1_arg!(__bs_call_charAt, "charAt", dummy_arr_1, crate::string::__bs_string_charAt);
crate::dispatch_1_arg!(__bs_call_charCodeAt, "charCodeAt", dummy_arr_1, crate::string::__bs_string_charCodeAt);
crate::dispatch_1_arg!(__bs_call_startsWith, "startsWith", dummy_arr_1, crate::string::__bs_string_startsWith);
crate::dispatch_1_arg!(__bs_call_endsWith, "endsWith", dummy_arr_1, crate::string::__bs_string_endsWith);
crate::dispatch_2_args!(__bs_call_substring, "substring", dummy_arr_2, crate::string::__bs_string_substring);
crate::dispatch_1_arg!(__bs_call_split, "split", dummy_arr_1, crate::string::__bs_string_split);
crate::dispatch_0_args!(__bs_call_trim, "trim", dummy_arr_0, crate::string::__bs_string_trim);
crate::dispatch_0_args!(__bs_call_toUpperCase, "toUpperCase", dummy_arr_0, crate::string::__bs_string_toUpperCase);
crate::dispatch_0_args!(__bs_call_toLowerCase, "toLowerCase", dummy_arr_0, crate::string::__bs_string_toLowerCase);
crate::dispatch_2_args!(__bs_call_replace, "replace", dummy_arr_2, crate::string::__bs_string_replace);
crate::dispatch_1_arg!(__bs_call_repeat, "repeat", dummy_arr_1, crate::string::__bs_string_repeat);
crate::dispatch_2_args!(__bs_call_padStart, "padStart", dummy_arr_2, crate::string::__bs_string_padStart);
crate::dispatch_2_args!(__bs_call_padEnd, "padEnd", dummy_arr_2, crate::string::__bs_string_padEnd);

// Date & Object Prototype Dispatchers
crate::dispatch_0_args!(__bs_call_getTime, "getTime", dummy_arr_0, dummy_str_0);
crate::dispatch_0_args!(__bs_call_getFullYear, "getFullYear", dummy_arr_0, dummy_str_0);
crate::dispatch_0_args!(__bs_call_getMonth, "getMonth", dummy_arr_0, dummy_str_0);
crate::dispatch_0_args!(__bs_call_getDate, "getDate", dummy_arr_0, dummy_str_0);
crate::dispatch_0_args!(__bs_call_getHours, "getHours", dummy_arr_0, dummy_str_0);
crate::dispatch_0_args!(__bs_call_getMinutes, "getMinutes", dummy_arr_0, dummy_str_0);
crate::dispatch_0_args!(__bs_call_getSeconds, "getSeconds", dummy_arr_0, dummy_str_0);
// __bs_call_toString is implemented in custom.rs (needs radix support for numbers)
crate::dispatch_0_args!(__bs_call_valueOf, "valueOf", dummy_arr_0, dummy_str_0);

