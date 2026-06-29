//! String runtime methods for BinScript.


const TAG_STRING: u64 = 0xFFF7_0000_0000_0000;
const TAG_MASK: u64 = 0xFFFF_0000_0000_0000;
const PAYLOAD_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;

/// Get the length of a tagged string.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_string_length(val: u64) -> u64 {
    let tag = val & TAG_MASK;
    if tag != TAG_STRING {
        return crate::circ::box_number(0.0);
    }
    let payload = val & PAYLOAD_MASK;
    if payload == 0 {
        return crate::circ::box_number(0.0);
    }
    let c_str = std::ffi::CStr::from_ptr(payload as *const libc::c_char);
    let len = c_str.to_bytes().len();
    crate::circ::box_number(len as f64)
}

/// `str.charAt(index)`
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_string_charAt(str_tagged: u64, index_tagged: u64) -> u64 {
    let s = crate::get_c_string_from_tagged(str_tagged);
    let idx = f64::from_bits(index_tagged) as i64;
    if idx < 0 || idx >= s.len() as i64 {
        return crate::create_tagged_string("");
    }
    let char_str = &s[idx as usize..(idx + 1) as usize];
    crate::create_tagged_string(char_str)
}

/// `str.charCodeAt(index)`
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_string_charCodeAt(str_tagged: u64, index_tagged: u64) -> u64 {
    let s = crate::get_c_string_from_tagged(str_tagged);
    let idx = f64::from_bits(index_tagged) as i64;
    if idx < 0 || idx >= s.len() as i64 {
        return crate::circ::box_number(f64::NAN);
    }
    let code = s.as_bytes()[idx as usize] as f64;
    crate::circ::box_number(code)
}

/// `str.startsWith(prefix)`
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_string_startsWith(str_tagged: u64, prefix_tagged: u64) -> u64 {
    let s = crate::get_c_string_from_tagged(str_tagged);
    let prefix = crate::get_c_string_from_tagged(prefix_tagged);
    if s.starts_with(prefix) {
        0xFFF4_0000_0000_0000 // true
    } else {
        0xFFF3_0000_0000_0000 // false
    }
}

/// `str.endsWith(suffix)`
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_string_endsWith(str_tagged: u64, suffix_tagged: u64) -> u64 {
    let s = crate::get_c_string_from_tagged(str_tagged);
    let suffix = crate::get_c_string_from_tagged(suffix_tagged);
    if s.ends_with(suffix) {
        0xFFF4_0000_0000_0000 // true
    } else {
        0xFFF3_0000_0000_0000 // false
    }
}

/// `str.substring(start, end)`
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_string_substring(str_tagged: u64, start_tagged: u64, end_tagged: u64) -> u64 {
    let s = crate::get_c_string_from_tagged(str_tagged);
    let len = s.len() as i64;
    
    let mut start = if start_tagged == 0xFFF1_0000_0000_0000 {
        0
    } else {
        let val = f64::from_bits(start_tagged);
        let val_int = if val.is_nan() { 0 } else { val as i64 };
        std::cmp::max(0, std::cmp::min(val_int, len))
    };
    
    let mut end = if end_tagged == 0xFFF1_0000_0000_0000 {
        len
    } else {
        let val = f64::from_bits(end_tagged);
        let val_int = if val.is_nan() { 0 } else { val as i64 };
        std::cmp::max(0, std::cmp::min(val_int, len))
    };
    
    if start > end {
        std::mem::swap(&mut start, &mut end);
    }
    
    let sub = &s[start as usize..end as usize];
    crate::create_tagged_string(sub)
}

/// `str.split(sep)`
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_string_split(str_tagged: u64, sep_tagged: u64) -> u64 {
    let s = crate::get_c_string_from_tagged(str_tagged);
    let sep = crate::get_c_string_from_tagged(sep_tagged);
    
    let arr_tagged = crate::array::__bs_array_new();
    let parts: Vec<&str> = s.split(sep).collect();
    for part in parts {
        let part_tagged = crate::create_tagged_string(part);
        crate::array::__bs_array_push(arr_tagged, part_tagged);
    }
    arr_tagged
}

/// `str.trim()`
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_string_trim(str_tagged: u64) -> u64 {
    let s = crate::get_c_string_from_tagged(str_tagged);
    crate::create_tagged_string(s.trim())
}

/// `str.toUpperCase()`
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_string_toUpperCase(str_tagged: u64) -> u64 {
    let s = crate::get_c_string_from_tagged(str_tagged);
    crate::create_tagged_string(&s.to_uppercase())
}

/// `str.toLowerCase()`
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_string_toLowerCase(str_tagged: u64) -> u64 {
    let s = crate::get_c_string_from_tagged(str_tagged);
    crate::create_tagged_string(&s.to_lowercase())
}

/// `str.replace(pattern, replacement)`
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_string_replace(str_tagged: u64, pattern_tagged: u64, replacement_tagged: u64) -> u64 {
    let s = crate::get_c_string_from_tagged(str_tagged);
    let pattern = crate::get_c_string_from_tagged(pattern_tagged);
    let replacement = crate::get_c_string_from_tagged(replacement_tagged);
    crate::create_tagged_string(&s.replacen(pattern, replacement, 1))
}

/// `str.repeat(count)`
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_string_repeat(str_tagged: u64, count_tagged: u64) -> u64 {
    let s = crate::get_c_string_from_tagged(str_tagged);
    let count = f64::from_bits(count_tagged) as usize;
    crate::create_tagged_string(&s.repeat(count))
}

/// `str.padStart(targetLength, padString)`
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_string_padStart(str_tagged: u64, target_len_tagged: u64, pad_str_tagged: u64) -> u64 {
    let s = crate::get_c_string_from_tagged(str_tagged);
    let target_len = f64::from_bits(target_len_tagged) as usize;
    if s.len() >= target_len {
        return str_tagged;
    }
    
    let pad = if pad_str_tagged == 0 {
        " "
    } else {
        crate::get_c_string_from_tagged(pad_str_tagged)
    };
    
    let pad_len = target_len - s.len();
    let mut prefix = pad.repeat(pad_len / pad.len() + 1);
    prefix.truncate(pad_len);
    
    crate::create_tagged_string(&format!("{}{}", prefix, s))
}

/// `str.padEnd(targetLength, padString)`
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_string_padEnd(str_tagged: u64, target_len_tagged: u64, pad_str_tagged: u64) -> u64 {
    let s = crate::get_c_string_from_tagged(str_tagged);
    let target_len = f64::from_bits(target_len_tagged) as usize;
    if s.len() >= target_len {
        return str_tagged;
    }
    
    let pad = if pad_str_tagged == 0 {
        " "
    } else {
        crate::get_c_string_from_tagged(pad_str_tagged)
    };
    
    let pad_len = target_len - s.len();
    let mut suffix = pad.repeat(pad_len / pad.len() + 1);
    suffix.truncate(pad_len);
    
    crate::create_tagged_string(&format!("{}{}", s, suffix))
}

/// `str.includes(searchString)`
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_string_includes(str_tagged: u64, search_tagged: u64) -> u64 {
    let s = crate::get_c_string_from_tagged(str_tagged);
    let search = crate::get_c_string_from_tagged(search_tagged);
    if s.contains(search) {
        0xFFF4_0000_0000_0000 // true
    } else {
        0xFFF3_0000_0000_0000 // false
    }
}
