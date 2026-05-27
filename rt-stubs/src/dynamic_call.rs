//! Dynamic method call dispatchers for BinScript.

use crate::gc;
use crate::VTable;

const TAG_ARRAY: u64 = 0xFFFB_0000_0000_0000;
const TAG_STRING: u64 = 0xFFF7_0000_0000_0000;
const TAG_OBJECT: u64 = 0xFFF6_0000_0000_0000;
const TAG_MASK: u64 = 0xFFFF_0000_0000_0000;
const PAYLOAD_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;

unsafe fn get_user_method(receiver: u64, idx: i32) -> Option<*const u8> {
    if idx < 0 { return None; }
    let payload = receiver & PAYLOAD_MASK;
    if payload == 0 { return None; }
    let obj_ptr = payload as *const *const VTable;
    let mut vtable = *obj_ptr;
    while !vtable.is_null() {
        let slot = *(vtable as *const *const u8).add(5 + idx as usize);
        if !slot.is_null() {
            return Some(slot);
        }
        vtable = (*vtable).parent;
    }
    None
}

/// Convert a NaN-boxed value to a display string.
unsafe fn value_to_string(val: u64) -> String {
    if val == 0 { return String::new(); } // undefined → ""
    let tag = val & TAG_MASK;
    if tag == 0xFFF3_0000_0000_0000 { return "false".to_string(); }
    if tag == 0xFFF4_0000_0000_0000 { return "true".to_string(); }
    if tag == 0xFFF5_0000_0000_0000 { return "null".to_string(); }
    if tag == 0xFFF7_0000_0000_0000 {
        let payload = val & PAYLOAD_MASK;
        if payload == 0 { return String::new(); }
        let c_str = unsafe { std::ffi::CStr::from_ptr(payload as *const libc::c_char) };
        return c_str.to_str().unwrap_or("").to_string();
    }
    // Number
    let f = f64::from_bits(val);
    if f == f.floor() && f.abs() < 1e15 {
        format!("{}", f as i64)
    } else {
        format!("{}", f)
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_index_get(obj: u64, index: u64) -> u64 {
    let tag = obj & TAG_MASK;
    if tag == TAG_ARRAY {
        crate::array::__bs_array_get(obj, index)
    } else {
        let prop_name = value_to_string(index);
        let prop_bytes = prop_name.as_bytes();
        crate::json_tape::__bs_prop_get(obj, prop_bytes.as_ptr(), prop_bytes.len() as u32)
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_index_set(obj: u64, index: u64, val: u64) {
    let tag = obj & TAG_MASK;
    if tag == TAG_ARRAY {
        crate::array::__bs_array_set(obj, index, val);
    } else {
        let prop_name = value_to_string(index);
        let prop_bytes = prop_name.as_bytes();
        crate::json_tape::__bs_prop_set(obj, prop_bytes.as_ptr(), prop_bytes.len() as u32, val);
    }
}

macro_rules! dispatch_0_args {
    ($name:ident, $method_name:expr, $arr_fn:expr, $str_fn:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(mut recv: u64, idx_boxed: u64) -> u64 {
            let idx = f64::from_bits(idx_boxed) as i32;
            let mut tag = recv & TAG_MASK;
            if tag == TAG_OBJECT {
                let payload = recv & PAYLOAD_MASK;
                if payload != 0 {
                    let obj_ptr = payload as *mut u8;
                    let vtable_ptr = *(obj_ptr as *const *const VTable);
                    if !vtable_ptr.is_null() {
                        let name_cstr = std::ffi::CStr::from_ptr((*vtable_ptr).name as *const libc::c_char);
                        let name_bytes = name_cstr.to_bytes();
                        if name_bytes == b"String" {
                            if let Some(prim) = crate::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        } else if name_bytes == b"Number" {
                            if let Some(prim) = crate::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        } else if name_bytes == b"Boolean" {
                            if let Some(prim) = crate::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        } else if name_bytes == b"Date" {
                            if $method_name == "getTime" || $method_name == "valueOf" {
                                return __bs_date_getTime(recv);
                            } else if $method_name == "getFullYear" {
                                return __bs_date_getFullYear(recv);
                            } else if $method_name == "getMonth" {
                                return __bs_date_getMonth(recv);
                            } else if $method_name == "getDate" {
                                return __bs_date_getDate(recv);
                            } else if $method_name == "getHours" {
                                return __bs_date_getHours(recv);
                            } else if $method_name == "getMinutes" {
                                return __bs_date_getMinutes(recv);
                            } else if $method_name == "getSeconds" {
                                return __bs_date_getSeconds(recv);
                            } else if $method_name == "toString" {
                                if let Some(prim) = crate::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                    let ms = f64::from_bits(prim);
                                    return crate::create_tagged_string(&date_to_string(ms));
                                }
                            }
                        }
                    }
                }
            }
            if $method_name == "toString" {
                if tag == TAG_STRING {
                    return recv;
                } else if tag == TAG_ARRAY {
                    return crate::__bs_String(recv);
                }
            } else if $method_name == "valueOf" {
                if tag == TAG_STRING || tag == TAG_ARRAY {
                    return recv;
                }
            }

            if tag == TAG_ARRAY {
                let f: unsafe extern "C" fn(u64) -> u64 = $arr_fn;
                f(recv)
            } else if tag == TAG_STRING {
                let f: unsafe extern "C" fn(u64) -> u64 = $str_fn;
                f(recv)
            } else if tag == TAG_OBJECT {
                if let Some(method_ptr) = get_user_method(recv, idx) {
                    let f: unsafe extern "C" fn(u64) -> u64 = std::mem::transmute(method_ptr);
                    f(recv)
                } else {
                    if $method_name == "toString" {
                        return crate::create_tagged_string("[object Object]");
                    }
                    if $method_name == "valueOf" {
                        return recv;
                    }
                    panic!("Method not found on user object");
                }
            } else {
                if $method_name == "toString" {
                    return crate::__bs_String(recv);
                } else if $method_name == "valueOf" {
                    return recv;
                }
                panic!("Method called on incompatible receiver type");
            }
        }
    };
}

macro_rules! dispatch_1_arg {
    ($name:ident, $method_name:expr, $arr_fn:expr, $str_fn:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(mut recv: u64, arg1: u64, idx_boxed: u64) -> u64 {
            let idx = f64::from_bits(idx_boxed) as i32;
            let mut tag = recv & TAG_MASK;
            if tag == TAG_OBJECT {
                let payload = recv & PAYLOAD_MASK;
                if payload != 0 {
                    let obj_ptr = payload as *mut u8;
                    let vtable_ptr = *(obj_ptr as *const *const VTable);
                    if !vtable_ptr.is_null() {
                        let name_cstr = std::ffi::CStr::from_ptr((*vtable_ptr).name as *const libc::c_char);
                        let name_bytes = name_cstr.to_bytes();
                        if name_bytes == b"String" {
                            if let Some(prim) = crate::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        } else if name_bytes == b"Number" {
                            if let Some(prim) = crate::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        } else if name_bytes == b"Boolean" {
                            if let Some(prim) = crate::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        }
                    }
                }
            }
            if tag == TAG_ARRAY {
                let f: unsafe extern "C" fn(u64, u64) -> u64 = $arr_fn;
                f(recv, arg1)
            } else if tag == TAG_STRING {
                let f: unsafe extern "C" fn(u64, u64) -> u64 = $str_fn;
                f(recv, arg1)
            } else if tag == TAG_OBJECT {
                if let Some(method_ptr) = get_user_method(recv, idx) {
                    let f: unsafe extern "C" fn(u64, u64) -> u64 = std::mem::transmute(method_ptr);
                    f(recv, arg1)
                } else {
                    panic!("Method not found on user object");
                }
            } else {
                panic!("Method called on incompatible receiver type");
            }
        }
    };
}

macro_rules! dispatch_2_args {
    ($name:ident, $method_name:expr, $arr_fn:expr, $str_fn:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(mut recv: u64, arg1: u64, arg2: u64, idx_boxed: u64) -> u64 {
            let idx = f64::from_bits(idx_boxed) as i32;
            let mut tag = recv & TAG_MASK;
            if tag == TAG_OBJECT {
                let payload = recv & PAYLOAD_MASK;
                if payload != 0 {
                    let obj_ptr = payload as *mut u8;
                    let vtable_ptr = *(obj_ptr as *const *const VTable);
                    if !vtable_ptr.is_null() {
                        let name_cstr = std::ffi::CStr::from_ptr((*vtable_ptr).name as *const libc::c_char);
                        let name_bytes = name_cstr.to_bytes();
                        if name_bytes == b"String" {
                            if let Some(prim) = crate::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        } else if name_bytes == b"Number" {
                            if let Some(prim) = crate::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        } else if name_bytes == b"Boolean" {
                            if let Some(prim) = crate::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        }
                    }
                }
            }
            if tag == TAG_ARRAY {
                let f: unsafe extern "C" fn(u64, u64, u64) -> u64 = $arr_fn;
                f(recv, arg1, arg2)
            } else if tag == TAG_STRING {
                let f: unsafe extern "C" fn(u64, u64, u64) -> u64 = $str_fn;
                f(recv, arg1, arg2)
            } else if tag == TAG_OBJECT {
                if let Some(method_ptr) = get_user_method(recv, idx) {
                    let f: unsafe extern "C" fn(u64, u64, u64) -> u64 = std::mem::transmute(method_ptr);
                    f(recv, arg1, arg2)
                } else {
                    panic!("Method not found on user object");
                }
            } else {
                panic!("Method called on incompatible receiver type");
            }
        }
    };
}

macro_rules! dispatch_3_args {
    ($name:ident, $method_name:expr, $arr_fn:expr, $str_fn:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(mut recv: u64, arg1: u64, arg2: u64, arg3: u64, idx_boxed: u64) -> u64 {
            let idx = f64::from_bits(idx_boxed) as i32;
            let mut tag = recv & TAG_MASK;
            if tag == TAG_OBJECT {
                let payload = recv & PAYLOAD_MASK;
                if payload != 0 {
                    let obj_ptr = payload as *mut u8;
                    let vtable_ptr = *(obj_ptr as *const *const VTable);
                    if !vtable_ptr.is_null() {
                        let name_cstr = std::ffi::CStr::from_ptr((*vtable_ptr).name as *const libc::c_char);
                        let name_bytes = name_cstr.to_bytes();
                        if name_bytes == b"String" {
                            if let Some(prim) = crate::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        } else if name_bytes == b"Number" {
                            if let Some(prim) = crate::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        } else if name_bytes == b"Boolean" {
                            if let Some(prim) = crate::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        }
                    }
                }
            }
            if tag == TAG_ARRAY {
                let f: unsafe extern "C" fn(u64, u64, u64, u64) -> u64 = $arr_fn;
                f(recv, arg1, arg2, arg3)
            } else if tag == TAG_STRING {
                let f: unsafe extern "C" fn(u64, u64, u64, u64) -> u64 = $str_fn;
                f(recv, arg1, arg2, arg3)
            } else if tag == TAG_OBJECT {
                if let Some(method_ptr) = get_user_method(recv, idx) {
                    let f: unsafe extern "C" fn(u64, u64, u64, u64) -> u64 = std::mem::transmute(method_ptr);
                    f(recv, arg1, arg2, arg3)
                } else {
                    panic!("Method not found on user object");
                }
            } else {
                panic!("Method called on incompatible receiver type");
            }
        }
    };
}

// Wrapper for forEach because it returns void
unsafe extern "C" fn array_forEach_wrapper(arr: u64, cb: u64) -> u64 {
    crate::array::__bs_array_forEach(arr, cb);
    0
}

// Non-existent string dummy fallback functions for methods only present on arrays
unsafe extern "C" fn dummy_str_0(_: u64) -> u64 { gc::box_number(0.0) }
unsafe extern "C" fn dummy_str_1(_: u64, _: u64) -> u64 { gc::box_number(0.0) }
unsafe extern "C" fn dummy_str_2(_: u64, _: u64, _: u64) -> u64 { gc::box_number(0.0) }
unsafe extern "C" fn dummy_str_3(_: u64, _: u64, _: u64, _: u64) -> u64 { gc::box_number(0.0) }

// Non-existent array dummy fallback functions for methods only present on strings
unsafe extern "C" fn dummy_arr_0(_: u64) -> u64 { 0 }
unsafe extern "C" fn dummy_arr_1(_: u64, _: u64) -> u64 { 0 }
unsafe extern "C" fn dummy_arr_2(_: u64, _: u64, _: u64) -> u64 { 0 }

// Array & String Dispatchers
dispatch_1_arg!(__bs_call_push, "push", crate::array::__bs_array_push, dummy_str_1);
dispatch_0_args!(__bs_call_pop, "pop", crate::array::__bs_array_pop, dummy_str_0);
dispatch_2_args!(__bs_call_slice, "slice", crate::array::__bs_array_slice, crate::string_methods::__bs_string_substring);
dispatch_1_arg!(__bs_call_includes, "includes", crate::array::__bs_array_includes, dummy_str_1);
dispatch_1_arg!(__bs_call_join, "join", crate::array::__bs_array_join, dummy_str_1);
dispatch_0_args!(__bs_call_reverse, "reverse", crate::array::__bs_array_reverse, dummy_str_0);
dispatch_1_arg!(__bs_call_concat, "concat", crate::array::__bs_array_concat, dummy_str_1);
dispatch_3_args!(__bs_call_fill, "fill", crate::array::__bs_array_fill, dummy_str_3);

dispatch_1_arg!(__bs_call_forEach, "forEach", array_forEach_wrapper, dummy_str_1);
dispatch_1_arg!(__bs_call_map, "map", crate::array::__bs_array_map, dummy_str_1);
dispatch_1_arg!(__bs_call_filter, "filter", crate::array::__bs_array_filter, dummy_str_1);
dispatch_1_arg!(__bs_call_find, "find", crate::array::__bs_array_find, dummy_str_1);
dispatch_1_arg!(__bs_call_findIndex, "findIndex", crate::array::__bs_array_findIndex, dummy_str_1);
dispatch_1_arg!(__bs_call_every, "every", crate::array::__bs_array_every, dummy_str_1);
dispatch_1_arg!(__bs_call_some, "some", crate::array::__bs_array_some, dummy_str_1);
dispatch_2_args!(__bs_call_reduce, "reduce", crate::array::__bs_array_reduce, dummy_str_2);

// String-only Dispatchers
dispatch_1_arg!(__bs_call_charAt, "charAt", dummy_arr_1, crate::string_methods::__bs_string_charAt);
dispatch_1_arg!(__bs_call_charCodeAt, "charCodeAt", dummy_arr_1, crate::string_methods::__bs_string_charCodeAt);
dispatch_1_arg!(__bs_call_startsWith, "startsWith", dummy_arr_1, crate::string_methods::__bs_string_startsWith);
dispatch_1_arg!(__bs_call_endsWith, "endsWith", dummy_arr_1, crate::string_methods::__bs_string_endsWith);
dispatch_2_args!(__bs_call_substring, "substring", dummy_arr_2, crate::string_methods::__bs_string_substring);
dispatch_1_arg!(__bs_call_split, "split", dummy_arr_1, crate::string_methods::__bs_string_split);
dispatch_0_args!(__bs_call_trim, "trim", dummy_arr_0, crate::string_methods::__bs_string_trim);
dispatch_0_args!(__bs_call_toUpperCase, "toUpperCase", dummy_arr_0, crate::string_methods::__bs_string_toUpperCase);
dispatch_0_args!(__bs_call_toLowerCase, "toLowerCase", dummy_arr_0, crate::string_methods::__bs_string_toLowerCase);
dispatch_2_args!(__bs_call_replace, "replace", dummy_arr_2, crate::string_methods::__bs_string_replace);
dispatch_1_arg!(__bs_call_repeat, "repeat", dummy_arr_1, crate::string_methods::__bs_string_repeat);
dispatch_2_args!(__bs_call_padStart, "padStart", dummy_arr_2, crate::string_methods::__bs_string_padStart);
dispatch_2_args!(__bs_call_padEnd, "padEnd", dummy_arr_2, crate::string_methods::__bs_string_padEnd);

// Date & Object Prototype Dispatchers
dispatch_0_args!(__bs_call_getTime, "getTime", dummy_arr_0, dummy_str_0);
dispatch_0_args!(__bs_call_getFullYear, "getFullYear", dummy_arr_0, dummy_str_0);
dispatch_0_args!(__bs_call_getMonth, "getMonth", dummy_arr_0, dummy_str_0);
dispatch_0_args!(__bs_call_getDate, "getDate", dummy_arr_0, dummy_str_0);
dispatch_0_args!(__bs_call_getHours, "getHours", dummy_arr_0, dummy_str_0);
dispatch_0_args!(__bs_call_getMinutes, "getMinutes", dummy_arr_0, dummy_str_0);
dispatch_0_args!(__bs_call_getSeconds, "getSeconds", dummy_arr_0, dummy_str_0);
dispatch_0_args!(__bs_call_toString, "toString", dummy_arr_0, dummy_str_0);
dispatch_0_args!(__bs_call_valueOf, "valueOf", dummy_arr_0, dummy_str_0);

// Custom dispatcher for indexOf (as both arrays and strings have it)
#[no_mangle]
pub unsafe extern "C" fn __bs_call_indexOf(mut recv: u64, search: u64, idx_boxed: u64) -> u64 {
    let idx = f64::from_bits(idx_boxed) as i32;
    let mut tag = recv & TAG_MASK;
    if tag == TAG_OBJECT {
        let payload = recv & PAYLOAD_MASK;
        if payload != 0 {
            let obj_ptr = payload as *mut u8;
            let vtable_ptr = *(obj_ptr as *const *const VTable);
            if !vtable_ptr.is_null() {
                let name_cstr = std::ffi::CStr::from_ptr((*vtable_ptr).name as *const libc::c_char);
                let name_bytes = name_cstr.to_bytes();
                if name_bytes == b"String" {
                    if let Some(prim) = crate::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                        recv = prim;
                        tag = recv & TAG_MASK;
                    }
                } else if name_bytes == b"Number" {
                    if let Some(prim) = crate::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                        recv = prim;
                        tag = recv & TAG_MASK;
                    }
                } else if name_bytes == b"Boolean" {
                    if let Some(prim) = crate::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                        recv = prim;
                        tag = recv & TAG_MASK;
                    }
                }
            }
        }
    }
    if tag == TAG_ARRAY {
        crate::array::__bs_array_indexOf(recv, search)
    } else if tag == TAG_STRING {
        let s = crate::get_c_string_from_tagged(recv);
        let pattern = crate::get_c_string_from_tagged(search);
        if let Some(pos) = s.find(pattern) {
            gc::box_number(pos as f64)
        } else {
            gc::box_number(-1.0)
        }
    } else if tag == TAG_OBJECT {
        if let Some(method_ptr) = get_user_method(recv, idx) {
            let f: unsafe extern "C" fn(u64, u64) -> u64 = std::mem::transmute(method_ptr);
            f(recv, search)
        } else {
            panic!("Method indexOf not found on object");
        }
    } else {
        panic!("Method indexOf called on incompatible receiver");
    }
}

// ===========================================================================
// Date Helpers & Prototype Implementations
// ===========================================================================

struct DateComponents {
    year: i32,
    month: u32,
    date: u32,
    hours: u32,
    minutes: u32,
    seconds: u32,
}

fn ms_to_components(ms: f64) -> DateComponents {
    let mut seconds = (ms / 1000.0).floor() as i64;
    let mut seconds_of_day = seconds % 86400;
    if seconds_of_day < 0 {
        seconds_of_day += 86400;
    }
    let hours = (seconds_of_day / 3600) as u32;
    let minutes = ((seconds_of_day % 3600) / 60) as u32;
    let seconds_ret = (seconds_of_day % 60) as u32;
    
    let mut days = seconds / 86400;
    if seconds % 86400 < 0 && seconds % 86400 != 0 {
        days -= 1;
    }
    
    let mut year = 1970;
    if days >= 0 {
        loop {
            let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
            let days_in_year = if leap { 366 } else { 365 };
            if days >= days_in_year {
                days -= days_in_year;
                year += 1;
            } else {
                break;
            }
        }
    } else {
        loop {
            year -= 1;
            let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
            let days_in_year = if leap { 366 } else { 365 };
            days += days_in_year;
            if days >= 0 {
                break;
            }
        }
    }
    
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let mut days_in_months = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if leap {
        days_in_months[1] = 29;
    }
    
    let mut month = 0;
    while days >= days_in_months[month] as i64 {
        days -= days_in_months[month] as i64;
        month += 1;
    }
    
    DateComponents {
        year,
        month: month as u32,
        date: (days + 1) as u32,
        hours,
        minutes,
        seconds: seconds_ret,
    }
}

pub(crate) fn date_to_string(ms: f64) -> String {
    let comps = ms_to_components(ms);
    let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let seconds = (ms / 1000.0).floor() as i64;
    let mut days = seconds / 86400;
    if seconds % 86400 < 0 && seconds % 86400 != 0 {
        days -= 1;
    }
    let mut wday = (days + 4) % 7;
    if wday < 0 {
        wday += 7;
    }
    let weekdays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    
    format!(
        "{} {} {:02} {} {:02}:{:02}:{:02} GMT",
        weekdays[wday as usize],
        months[comps.month as usize],
        comps.date,
        comps.year,
        comps.hours,
        comps.minutes,
        comps.seconds
    )
}

#[no_mangle]
pub unsafe extern "C" fn __bs_date_getTime(recv: u64) -> u64 {
    let tag = recv & TAG_MASK;
    if tag == TAG_OBJECT {
        let payload = recv & PAYLOAD_MASK;
        let obj_ptr = payload as *mut u8;
        if let Some(prim) = crate::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
            return prim;
        }
    }
    gc::box_number(0.0)
}

#[no_mangle]
pub unsafe extern "C" fn __bs_date_getFullYear(recv: u64) -> u64 {
    let ms = f64::from_bits(__bs_date_getTime(recv));
    gc::box_number(ms_to_components(ms).year as f64)
}

#[no_mangle]
pub unsafe extern "C" fn __bs_date_getMonth(recv: u64) -> u64 {
    let ms = f64::from_bits(__bs_date_getTime(recv));
    gc::box_number(ms_to_components(ms).month as f64)
}

#[no_mangle]
pub unsafe extern "C" fn __bs_date_getDate(recv: u64) -> u64 {
    let ms = f64::from_bits(__bs_date_getTime(recv));
    gc::box_number(ms_to_components(ms).date as f64)
}

#[no_mangle]
pub unsafe extern "C" fn __bs_date_getHours(recv: u64) -> u64 {
    let ms = f64::from_bits(__bs_date_getTime(recv));
    gc::box_number(ms_to_components(ms).hours as f64)
}

#[no_mangle]
pub unsafe extern "C" fn __bs_date_getMinutes(recv: u64) -> u64 {
    let ms = f64::from_bits(__bs_date_getTime(recv));
    gc::box_number(ms_to_components(ms).minutes as f64)
}

#[no_mangle]
pub unsafe extern "C" fn __bs_date_getSeconds(recv: u64) -> u64 {
    let ms = f64::from_bits(__bs_date_getTime(recv));
    gc::box_number(ms_to_components(ms).seconds as f64)
}

#[no_mangle]
pub unsafe extern "C" fn __bs_call_apply(callee: u64, _this_val: u64, args_array: u64) -> u64 {
    let tag = callee & TAG_MASK;
    if tag != 0xFFF9_0000_0000_0000 {
        panic!("__bs_call_apply: callee is not a closure (tag: {:X})", tag);
    }
    
    let closure_ptr = (callee & PAYLOAD_MASK) as *const u64;
    let fn_ptr = *closure_ptr;
    if fn_ptr == 0 {
        panic!("__bs_call_apply: closure has null function pointer");
    }
    
    let len_boxed = crate::array::__bs_array_length(args_array);
    let len = f64::from_bits(len_boxed) as usize;
    let mut args = Vec::new();
    for i in 0..len {
        let idx = gc::box_number(i as f64);
        args.push(crate::array::__bs_array_get(args_array, idx));
    }

    match len {
        0 => {
            let cb: unsafe extern "C" fn(u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(callee)
        }
        1 => {
            let cb: unsafe extern "C" fn(u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(callee, args[0])
        }
        2 => {
            let cb: unsafe extern "C" fn(u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(callee, args[0], args[1])
        }
        3 => {
            let cb: unsafe extern "C" fn(u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(callee, args[0], args[1], args[2])
        }
        4 => {
            let cb: unsafe extern "C" fn(u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(callee, args[0], args[1], args[2], args[3])
        }
        5 => {
            let cb: unsafe extern "C" fn(u64, u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(callee, args[0], args[1], args[2], args[3], args[4])
        }
        6 => {
            let cb: unsafe extern "C" fn(u64, u64, u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(callee, args[0], args[1], args[2], args[3], args[4], args[5])
        }
        7 => {
            let cb: unsafe extern "C" fn(u64, u64, u64, u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(callee, args[0], args[1], args[2], args[3], args[4], args[5], args[6])
        }
        8 => {
            let cb: unsafe extern "C" fn(u64, u64, u64, u64, u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(callee, args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7])
        }
        _ => {
            panic!("__bs_call_apply: dynamic call with {} arguments is unsupported (max 8)", len);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_vcall_apply(obj: u64, method_idx_val: u64, args_array: u64) -> u64 {
    let method_idx = f64::from_bits(method_idx_val) as i32;
    let payload = obj & PAYLOAD_MASK;
    if payload == 0 {
        panic!("__bs_vcall_apply: obj is null");
    }
    
    let obj_ptr = payload as *const *const VTable;
    let vtable_ptr = *obj_ptr;
    if vtable_ptr.is_null() {
        panic!("__bs_vcall_apply: vtable is null");
    }
    
    // Look up method pointer in the vtable chain
    let mut current_vtable = vtable_ptr;
    let mut fn_ptr: *const u8 = std::ptr::null();
    while !current_vtable.is_null() {
        let slot = *(current_vtable as *const *const u8).add(5 + method_idx as usize);
        if !slot.is_null() {
            fn_ptr = slot;
            break;
        }
        current_vtable = (*current_vtable).parent;
    }
    if fn_ptr.is_null() {
        panic!("__bs_vcall_apply: method not found in vtable (idx: {})", method_idx);
    }
        let len_boxed = crate::array::__bs_array_length(args_array);
    let len = f64::from_bits(len_boxed) as usize;
    let mut args = Vec::new();
    for i in 0..len {
        let idx = gc::box_number(i as f64);
        args.push(crate::array::__bs_array_get(args_array, idx));
    }

    // Note: class methods expect (this, arg1, arg2...)
    // Since args_array already has `obj` prepended as its first element,
    // args[0] is the receiver `this`, and args[1..] are the subsequent arguments.
    match len {
        1 => {
            let cb: unsafe extern "C" fn(u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(args[0])
        }
        2 => {
            let cb: unsafe extern "C" fn(u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(args[0], args[1])
        }
        3 => {
            let cb: unsafe extern "C" fn(u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(args[0], args[1], args[2])
        }
        4 => {
            let cb: unsafe extern "C" fn(u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(args[0], args[1], args[2], args[3])
        }
        5 => {
            let cb: unsafe extern "C" fn(u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(args[0], args[1], args[2], args[3], args[4])
        }
        6 => {
            let cb: unsafe extern "C" fn(u64, u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(args[0], args[1], args[2], args[3], args[4], args[5])
        }
        7 => {
            let cb: unsafe extern "C" fn(u64, u64, u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(args[0], args[1], args[2], args[3], args[4], args[5], args[6])
        }
        8 => {
            let cb: unsafe extern "C" fn(u64, u64, u64, u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7])
        }
        9 => {
            let cb: unsafe extern "C" fn(u64, u64, u64, u64, u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7], args[8])
        }
        _ => {
            panic!("__bs_vcall_apply: dynamic call with {} arguments is unsupported (max 8 actual args)", len - 1);
        }
    }
}
