use super::string_utils::{get_c_string_from_tagged, create_tagged_string};
use crate::core::vtable::VTable;

use crate::objects::dynamic_props::get_dynamic_property;
use crate::objects::builtins::{
    __bs_String_new_1, __bs_Boolean_new_1, __bs_Number_new_1, __bs_new_object,
};

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_String(val: u64) -> u64 {
    let tag = val & 0xFFFF_0000_0000_0000;
    if tag == 0xFFF1_0000_0000_0000 {
        create_tagged_string("undefined")
    } else if tag == 0xFFF2_0000_0000_0000 {
        create_tagged_string("null")
    } else if tag == 0xFFF3_0000_0000_0000 {
        create_tagged_string("false")
    } else if tag == 0xFFF4_0000_0000_0000 {
        create_tagged_string("true")
    } else if tag == 0xFFF7_0000_0000_0000 {
        val
    } else if tag == 0xFFF8_0000_0000_0000 {
        // Symbol -> "Symbol(description)" or "Symbol()"
        let payload = val & 0x0000_FFFF_FFFF_FFFF;
        let block = payload as *const u64;
        let desc_ptr = *(block.add(1)) as *const u8;
        if desc_ptr.is_null() {
            create_tagged_string("Symbol()")
        } else {
            let c_str = std::ffi::CStr::from_ptr(desc_ptr as *const libc::c_char);
            let desc = c_str.to_str().unwrap_or("");
            create_tagged_string(&format!("Symbol({})", desc))
        }
    } else if tag == 0xFFF6_0000_0000_0000 {
        let payload = val & 0x0000_FFFF_FFFF_FFFF;
        let obj_ptr = payload as *mut u8;
        let vtable_ptr = *(obj_ptr as *const *const VTable);
        if !vtable_ptr.is_null() {
            let name_cstr = std::ffi::CStr::from_ptr((*vtable_ptr).name as *const libc::c_char);
            let name_bytes = name_cstr.to_bytes();
            if name_bytes == b"String" || name_bytes == b"Number" || name_bytes == b"Boolean" || name_bytes == b"Date" {
                if let Some(prim) = get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                    return __bs_String(prim);
                }
            }
        }
        create_tagged_string("[object Object]")
    } else if tag == 0xFFFB_0000_0000_0000 {
        let len_boxed = crate::array::__bs_array_length(val);
        let len = f64::from_bits(len_boxed) as usize;
        let mut parts = Vec::new();
        for i in 0..len {
            let elem = crate::array::__bs_array_get(val, crate::circ::box_number(i as f64));
            let s_elem_tagged = __bs_String(elem);
            let s_elem = get_c_string_from_tagged(s_elem_tagged);
            parts.push(s_elem.to_string());
        }
        create_tagged_string(&parts.join(","))
    } else {
        let f = f64::from_bits(val);
        let s = if f.is_nan() {
            "NaN".to_string()
        } else if f.is_infinite() {
            if f.is_sign_positive() { "Infinity".to_string() } else { "-Infinity".to_string() }
        } else if f == f.floor() && f.abs() < 1e15 {
            format!("{}", f as i64)
        } else {
            format!("{}", f)
        };
        create_tagged_string(&s)
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_Number(val: u64) -> u64 {
    let tag = val & 0xFFFF_0000_0000_0000;
    let num = if tag == 0xFFF1_0000_0000_0000 {
        f64::NAN
    } else if tag == 0xFFF2_0000_0000_0000 {
        0.0
    } else if tag == 0xFFF3_0000_0000_0000 {
        0.0
    } else if tag == 0xFFF4_0000_0000_0000 {
        1.0
    } else if tag == 0xFFF7_0000_0000_0000 {
        let s = get_c_string_from_tagged(val);
        s.trim().parse::<f64>().unwrap_or(f64::NAN)
    } else if tag == 0xFFF6_0000_0000_0000 {
        let payload = val & 0x0000_FFFF_FFFF_FFFF;
        let obj_ptr = payload as *mut u8;
        let vtable_ptr = *(obj_ptr as *const *const VTable);
        if !vtable_ptr.is_null() {
            let name_cstr = std::ffi::CStr::from_ptr((*vtable_ptr).name as *const libc::c_char);
            let name_bytes = name_cstr.to_bytes();
            if name_bytes == b"String" || name_bytes == b"Number" || name_bytes == b"Boolean" || name_bytes == b"Date" {
                if let Some(prim) = get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                    return __bs_Number(prim);
                }
            }
        }
        f64::NAN
    } else if tag == 0xFFFB_0000_0000_0000 {
        let s_tagged = __bs_String(val);
        let s = get_c_string_from_tagged(s_tagged);
        s.trim().parse::<f64>().unwrap_or(f64::NAN)
    } else {
        return val;
    };
    crate::circ::box_number(num)
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_Boolean(val: u64) -> u64 {
    let tag = val & 0xFFFF_0000_0000_0000;
    let b = if tag == 0xFFF1_0000_0000_0000 {
        false
    } else if tag == 0xFFF2_0000_0000_0000 {
        false
    } else if tag == 0xFFF3_0000_0000_0000 {
        false
    } else if tag == 0xFFF4_0000_0000_0000 {
        true
    } else if tag == 0xFFF7_0000_0000_0000 {
        let s = get_c_string_from_tagged(val);
        !s.is_empty()
    } else if tag == 0xFFF8_0000_0000_0000 {
        true // symbols are always truthy
    } else if tag == 0xFFF6_0000_0000_0000 || tag == 0xFFFB_0000_0000_0000 || tag == 0xFFF9_0000_0000_0000 || tag == 0xFFFA_0000_0000_0000 || tag == 0xFFFC_0000_0000_0000 || tag == 0xFFFD_0000_0000_0000 {
        true
    } else {
        let f = f64::from_bits(val);
        f != 0.0 && !f.is_nan()
    };
    if b { 0xFFF4_0000_0000_0000 } else { 0xFFF3_0000_0000_0000 }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_Object(val: u64) -> u64 {
    let tag = val & 0xFFFF_0000_0000_0000;
    if tag == 0xFFF1_0000_0000_0000 || tag == 0xFFF2_0000_0000_0000 {
        __bs_new_object()
    } else if tag == 0xFFF6_0000_0000_0000 || tag == 0xFFFB_0000_0000_0000 || tag == 0xFFF9_0000_0000_0000 || tag == 0xFFFA_0000_0000_0000 || tag == 0xFFFC_0000_0000_0000 || tag == 0xFFFD_0000_0000_0000 {
        val
    } else if tag == 0xFFF7_0000_0000_0000 {
        __bs_String_new_1(val)
    } else if tag == 0xFFF3_0000_0000_0000 || tag == 0xFFF4_0000_0000_0000 {
        __bs_Boolean_new_1(val)
    } else {
        __bs_Number_new_1(val)
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_Date(_val: u64) -> u64 {
    let now = std::time::SystemTime::now();
    let since_the_epoch = now.duration_since(std::time::UNIX_EPOCH).expect("Time went backwards");
    let ms = since_the_epoch.as_millis() as f64;
    create_tagged_string(&crate::objects::date::date_to_string(ms))
}
