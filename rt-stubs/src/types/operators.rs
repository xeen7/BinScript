use crate::core::vtable::VTable;
use crate::types::string_utils::{get_c_string_from_tagged, create_tagged_string};
use crate::types::coercion::{__bs_String, __bs_Number};
use crate::objects::dynamic_props::{get_dynamic_property, delete_dynamic_property};

#[no_mangle]
pub unsafe extern "C" fn __bs_strict_eq(l: u64, r: u64) -> u64 {
    const TAG_STRING: u64 = 0xFFF7_0000_0000_0000;
    const TAG_MIN: u64 = 0xFFF1;
    const TAG_TRUE: u64 = 0xFFF4_0000_0000_0000;
    const TAG_FALSE: u64 = 0xFFF3_0000_0000_0000;

    let l_tag = l & 0xFFFF_0000_0000_0000;
    let r_tag = r & 0xFFFF_0000_0000_0000;

    // Check if either is a plain f64 number.
    // Top 16 bits of a number are < TAG_MIN (0xFFF1).
    let l_is_num = (l >> 48) < TAG_MIN;
    let r_is_num = (r >> 48) < TAG_MIN;

    if l_is_num && r_is_num {
        let lf = f64::from_bits(l);
        let rf = f64::from_bits(r);
        // Special check: NaN === NaN is false in JS
        if lf.is_nan() || rf.is_nan() {
            TAG_FALSE
        } else if lf == rf {
            TAG_TRUE
        } else {
            TAG_FALSE
        }
    } else if !l_is_num && !r_is_num {
        if l_tag == TAG_STRING && r_tag == TAG_STRING {
            let ls = get_c_string_from_tagged(l);
            let rs = get_c_string_from_tagged(r);
            if ls == rs {
                TAG_TRUE
            } else {
                TAG_FALSE
            }
        } else {
            if l == r {
                TAG_TRUE
            } else {
                TAG_FALSE
            }
        }
    } else {
        TAG_FALSE
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_strict_ne(l: u64, r: u64) -> u64 {
    const TAG_TRUE: u64 = 0xFFF4_0000_0000_0000;
    const TAG_FALSE: u64 = 0xFFF3_0000_0000_0000;
    if __bs_strict_eq(l, r) == TAG_TRUE {
        TAG_FALSE
    } else {
        TAG_TRUE
    }
}

/// JS `+` operator: numeric addition when both sides are numbers,
/// string concatenation when either side is a string (or coerces to one).
#[no_mangle]
pub unsafe extern "C" fn __bs_add(l: u64, r: u64) -> u64 {
    const TAG_STRING: u64 = 0xFFF7_0000_0000_0000;
    const TAG_MIN: u64    = 0xFFF1;

    let l_is_num = (l >> 48) < TAG_MIN;
    let r_is_num = (r >> 48) < TAG_MIN;

    let l_tag = l & 0xFFFF_0000_0000_0000;
    let r_tag = r & 0xFFFF_0000_0000_0000;

    let l_is_str = l_tag == TAG_STRING;
    let r_is_str = r_tag == TAG_STRING;

    if l_is_num && r_is_num {
        // Both plain numbers — float addition
        let lf = f64::from_bits(l);
        let rf = f64::from_bits(r);
        (lf + rf).to_bits()
    } else if l_is_str || r_is_str {
        // At least one string — coerce both and concatenate
        let ls = get_c_string_from_tagged(__bs_String(l));
        let rs = get_c_string_from_tagged(__bs_String(r));
        let concat = format!("{}{}", ls, rs);
        create_tagged_string(&concat)
    } else {
        // Both are non-string tagged values (null, bool, undefined, object) —
        // coerce to number via f64 bitcast as a best-effort fallback
        let lf = f64::from_bits(l);
        let rf = f64::from_bits(r);
        (lf + rf).to_bits()
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_is_nullish(val: u64) -> u64 {
    const TAG_TRUE: u64 = 0xFFF4_0000_0000_0000;
    const TAG_FALSE: u64 = 0xFFF3_0000_0000_0000;
    let tag = val & 0xFFFF_0000_0000_0000;
    if tag == 0xFFF1_0000_0000_0000 || tag == 0xFFF2_0000_0000_0000 {
        TAG_TRUE
    } else {
        TAG_FALSE
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_exp(l: u64, r: u64) -> u64 {
    let lf = f64::from_bits(__bs_Number(l));
    let rf = f64::from_bits(__bs_Number(r));
    lf.powf(rf).to_bits()
}

#[no_mangle]
pub unsafe extern "C" fn __bs_in(key: u64, obj: u64) -> u64 {
    const TAG_TRUE: u64 = 0xFFF4_0000_0000_0000;
    const TAG_FALSE: u64 = 0xFFF3_0000_0000_0000;
    
    let key_str_tagged = __bs_String(key);
    let key_str = get_c_string_from_tagged(key_str_tagged);

    let tag = obj & 0xFFFF_0000_0000_0000;
    if tag == 0xFFF6_0000_0000_0000 {
        let payload = obj & 0x0000_FFFF_FFFF_FFFF;
        let obj_ptr = payload as *mut u8;
        
        if get_dynamic_property(obj_ptr, key_str).is_some() {
            return TAG_TRUE;
        }

        let vtable_ptr = *(obj_ptr as *const *const VTable);
        if !vtable_ptr.is_null() {
            // Check VTable field names
            let mut i = 0;
            while i < (*vtable_ptr).fields_count {
                let name_cstr = std::ffi::CStr::from_ptr(*(*vtable_ptr).field_names.add(i as usize) as *const libc::c_char);
                if name_cstr.to_str().unwrap_or("") == key_str {
                    return TAG_TRUE;
                }
                i += 1;
            }
        }
    }
    TAG_FALSE
}

#[no_mangle]
pub unsafe extern "C" fn __bs_delete_prop(obj: u64, key: u64) -> u64 {
    const TAG_TRUE: u64 = 0xFFF4_0000_0000_0000;
    
    let key_str_tagged = __bs_String(key);
    let key_str = get_c_string_from_tagged(key_str_tagged);

    let tag = obj & 0xFFFF_0000_0000_0000;
    if tag == 0xFFF6_0000_0000_0000 {
        let payload = obj & 0x0000_FFFF_FFFF_FFFF;
        let obj_ptr = payload as *mut u8;
        delete_dynamic_property(obj_ptr, key_str);
    }
    TAG_TRUE
}
