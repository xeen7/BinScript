use crate::VTable;
use crate::dynamic_call::helpers::{TAG_MASK, TAG_ARRAY, TAG_STRING, TAG_OBJECT, PAYLOAD_MASK, get_user_method};

/// Custom dispatcher for `toString(radix?)`.
/// - Numbers: format with the given radix (default 10). (255).toString(16) → "ff"
/// - Strings: return as-is.
/// - Arrays: coerce via __bs_String.
/// - Objects: forward to vtable/user method.
/// Codegen declares this as dispatch_1 signature: (recv, arg, idx) -> u64
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_call_toString(mut recv: u64, arg: u64, idx_boxed: u64) -> u64 {
    let idx = f64::from_bits(idx_boxed) as i32;
    let mut tag = recv & TAG_MASK;

    // Unwrap Number/String/Boolean wrapper objects first
    if tag == TAG_OBJECT {
        let payload = recv & PAYLOAD_MASK;
        if payload != 0 {
            let obj_ptr = payload as *mut u8;
            let vtable_ptr = *(obj_ptr as *const *const VTable);
            if !vtable_ptr.is_null() {
                let name_cstr = std::ffi::CStr::from_ptr((*vtable_ptr).name as *const libc::c_char);
                let name_bytes = name_cstr.to_bytes();
                if name_bytes == b"Number" || name_bytes == b"String" || name_bytes == b"Boolean" {
                    if let Some(prim) = crate::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                        recv = prim;
                        tag = recv & TAG_MASK;
                    }
                } else {
                    // User object — forward to vtable method or fallback
                    if let Some(method_ptr) = get_user_method(recv, idx) {
                        let f: unsafe extern "C-unwind" fn(u64, u64) -> u64 = std::mem::transmute(method_ptr);
                        return f(recv, arg);
                    }
                    return crate::types::string_utils::create_tagged_string("[object Object]");
                }
            }
        }
    }

    // Determine radix: arg is undefined (0xFFF1…) or NaN means use 10
    let radix = {
        let raw_tag = arg & TAG_MASK;
        if raw_tag >= 0xFFF1_0000_0000_0000 {
            10u32 // undefined/null/bool → default 10
        } else {
            let r = f64::from_bits(arg) as u32;
            if r < 2 || r > 36 { 10 } else { r }
        }
    };

    // Raw number: format with radix
    if tag < 0xFFF1_0000_0000_0000 {
        let val = f64::from_bits(recv);
        let s = if radix == 10 {
            // Use standard number-to-string formatting
            if val == val.floor() && val.abs() < 1e15 {
                format!("{}", val as i64)
            } else {
                format!("{}", val)
            }
        } else {
            // Integer radix conversion
            let int_val = val as i64;
            match radix {
                2  => format!("{:b}", int_val),
                8  => format!("{:o}", int_val),
                16 => format!("{:x}", int_val),
                _  => {
                    // Generic radix via digit-by-digit
                    if int_val == 0 { "0".to_string() }
                    else {
                        const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
                        let negative = int_val < 0;
                        let mut n = int_val.unsigned_abs();
                        let mut chars = Vec::new();
                        while n > 0 {
                            chars.push(DIGITS[(n % radix as u64) as usize] as char);
                            n /= radix as u64;
                        }
                        if negative { chars.push('-'); }
                        chars.iter().rev().collect()
                    }
                }
            }
        };
        return crate::types::string_utils::create_tagged_string(&s);
    }

    // String: return as-is
    if tag == TAG_STRING {
        return recv;
    }

    // Array: coerce to string
    if (recv & TAG_MASK) == TAG_ARRAY || (recv & TAG_MASK) == 0x7FFB_0000_0000_0000 || (recv & TAG_MASK) == 0x7FFA_0000_0000_0000 {
        return crate::types::coercion::__bs_String(recv);
    }

    // Fallback
    crate::types::string_utils::create_tagged_string("[object Object]")
}

// Custom dispatcher for indexOf (as both arrays and strings have it)
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_call_indexOf(mut recv: u64, search: u64, idx_boxed: u64) -> u64 {
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
    if (recv & TAG_MASK) == TAG_ARRAY || (recv & TAG_MASK) == 0x7FFB_0000_0000_0000 || (recv & TAG_MASK) == 0x7FFA_0000_0000_0000 {
        crate::array::__bs_array_indexOf(recv, search)
    } else if tag == TAG_STRING {
        let s = crate::get_c_string_from_tagged(recv);
        let pattern = crate::get_c_string_from_tagged(search);
        if let Some(pos) = s.find(pattern) {
            crate::circ::box_number(pos as f64)
        } else {
            crate::circ::box_number(-1.0)
        }
    } else if tag == TAG_OBJECT {
        if let Some(method_ptr) = get_user_method(recv, idx) {
            let f: unsafe extern "C-unwind" fn(u64, u64) -> u64 = std::mem::transmute(method_ptr);
            f(recv, search)
        } else {
            panic!("Method indexOf not found on object");
        }
    } else {
        panic!("Method indexOf called on incompatible receiver");
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_call_next(recv: u64, arg1: u64, idx_boxed: u64) -> u64 {
    let idx = f64::from_bits(idx_boxed) as i32;
    let tag = recv & TAG_MASK;
    
    // Check if it's a generator iterator
    if tag == 0xFFFA_0000_0000_0000 {
        let is_done_val = crate::__bs_generator_is_done(recv);
        let done = is_done_val == 0xFFF4_0000_0000_0000;
        
        let value = if done {
            // Already done, return undefined
            0xFFF1_0000_0000_0000
        } else {
            crate::__bs_generator_next(recv, arg1)
        };
        
        // Re-check done status because generator might have just completed
        let is_done_val_after = crate::__bs_generator_is_done(recv);
        
        let result_obj = crate::__bs_alloc(&crate::core::vtable::GENERATOR_RESULT_VTABLE, 32);
        let payload = result_obj & PAYLOAD_MASK;
        let obj_ptr = payload as *mut u8;
        
        // Zero out the dynamic props slot
        let slot_props = obj_ptr.add(8) as *mut u64;
        *slot_props = 0;
        
        // Write 'value' to first class field (offset 16)
        let slot_value = obj_ptr.add(16) as *mut u64;
        *slot_value = value;
        
        // Write 'done' to second class field (offset 24)
        let slot_done = obj_ptr.add(24) as *mut u64;
        *slot_done = is_done_val_after;
        
        // println!("__bs_call_next returning value for {:x}: {:x}", recv, value);
        
        return result_obj;
    }
    
    if tag == TAG_OBJECT {
        if let Some(method_ptr) = get_user_method(recv, idx) {
            let f: unsafe extern "C-unwind" fn(u64, u64) -> u64 = std::mem::transmute(method_ptr);
            f(recv, arg1)
        } else {
            panic!("Method next not found on user object");
        }
    } else {
        panic!("Method next called on incompatible receiver type");
    }
}

/// Helper: unbox a NaN-boxed value to f64.
/// Works for raw numbers and Number wrapper objects.
unsafe fn unbox_number(recv: u64) -> Option<f64> {
    let tag = recv & TAG_MASK;
    // Raw float: top 16 bits are NOT in the special tagged range (< 0xFFF1)
    if tag < 0xFFF1_0000_0000_0000 {
        return Some(f64::from_bits(recv));
    }
    // Number wrapper object: TAG_OBJECT with vtable name "Number"
    if tag == TAG_OBJECT {
        let payload = recv & PAYLOAD_MASK;
        if payload != 0 {
            let obj_ptr = payload as *mut u8;
            let vtable_ptr = *(obj_ptr as *const *const VTable);
            if !vtable_ptr.is_null() {
                let name_cstr = std::ffi::CStr::from_ptr((*vtable_ptr).name as *const libc::c_char);
                if name_cstr.to_bytes() == b"Number" {
                    if let Some(prim) = crate::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                        return Some(f64::from_bits(prim));
                    }
                }
            }
        }
    }
    None
}

/// `(number).toFixed(digits)` — formats a number with a fixed number of decimal places.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_call_toFixed(recv: u64, digits_tagged: u64, _idx_boxed: u64) -> u64 {
    let digits = f64::from_bits(digits_tagged) as usize;
    if let Some(val) = unbox_number(recv) {
        let s = format!("{:.prec$}", val, prec = digits);
        return crate::types::string_utils::create_tagged_string(&s);
    }
    // Fallback: user object with a custom toFixed method
    let idx = f64::from_bits(_idx_boxed) as i32;
    if recv & TAG_MASK == TAG_OBJECT {
        if let Some(method_ptr) = get_user_method(recv, idx) {
            let f: unsafe extern "C-unwind" fn(u64, u64) -> u64 = std::mem::transmute(method_ptr);
            return f(recv, digits_tagged);
        }
    }
    crate::types::string_utils::create_tagged_string("NaN")
}

/// `(number).toPrecision(precision)` — formats a number to a given number of significant digits.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_call_toPrecision(recv: u64, precision_tagged: u64, _idx_boxed: u64) -> u64 {
    let precision = f64::from_bits(precision_tagged) as usize;
    if let Some(val) = unbox_number(recv) {
        let s = if precision == 0 || val == 0.0 {
            if precision <= 1 {
                "0".to_string()
            } else {
                format!("0.{}", "0".repeat(precision - 1))
            }
        } else {
            // Compute how many decimal places we need for `precision` significant digits
            let magnitude = val.abs().log10().floor() as i32;
            let decimal_places = ((precision as i32) - magnitude - 1).max(0) as usize;
            format!("{:.prec$}", val, prec = decimal_places)
        };
        return crate::types::string_utils::create_tagged_string(&s);
    }
    // Fallback: user object with a custom toPrecision method
    let idx = f64::from_bits(_idx_boxed) as i32;
    if recv & TAG_MASK == TAG_OBJECT {
        if let Some(method_ptr) = get_user_method(recv, idx) {
            let f: unsafe extern "C-unwind" fn(u64, u64) -> u64 = std::mem::transmute(method_ptr);
            return f(recv, precision_tagged);
        }
    }
    crate::types::string_utils::create_tagged_string("NaN")
}





