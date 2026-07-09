const TAG_MASK: u64 = 0xFFFF_0000_0000_0000;
const TAG_STRING: u64 = 0xFFF7_0000_0000_0000;

// --- Global Functions ---

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_parseInt(s_tagged: u64, radix_tagged: u64) -> u64 {
    if radix_tagged == 0xFFF1_0000_0000_0000 {
        __bs_parseInt_1(s_tagged)
    } else {
        __bs_parseInt_2(s_tagged, radix_tagged)
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_parseInt_1(s_tagged: u64) -> u64 {
    let s = crate::get_c_string_from_tagged(s_tagged);
    let trimmed = s.trim_start();
    
    let mut end = 0;
    let mut chars = trimmed.chars().peekable();
    if let Some(&c) = chars.peek() {
        if c == '+' || c == '-' {
            end += 1;
            chars.next();
        }
    }
    while let Some(&c) = chars.peek() {
        if c.is_digit(10) {
            end += 1;
            chars.next();
        } else {
            break;
        }
    }
    
    let prefix = &trimmed[..end];
    if let Ok(val) = i64::from_str_radix(prefix, 10) {
        crate::circ::box_number(val as f64)
    } else {
        std::f64::NAN.to_bits()
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_parseInt_2(s_tagged: u64, radix_tagged: u64) -> u64 {
    let s = crate::get_c_string_from_tagged(s_tagged);
    let radix = f64::from_bits(radix_tagged) as u32;
    if radix < 2 || radix > 36 {
        return std::f64::NAN.to_bits();
    }
    let trimmed = s.trim_start();
    
    let mut end = 0;
    let mut chars = trimmed.chars().peekable();
    if let Some(&c) = chars.peek() {
        if c == '+' || c == '-' {
            end += 1;
            chars.next();
        }
    }
    while let Some(&c) = chars.peek() {
        if c.is_digit(radix) {
            end += 1;
            chars.next();
        } else {
            break;
        }
    }
    
    let prefix = &trimmed[..end];
    if let Ok(val) = i64::from_str_radix(prefix, radix) {
        crate::circ::box_number(val as f64)
    } else {
        std::f64::NAN.to_bits()
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_parseFloat(s_tagged: u64) -> u64 {
    let s = crate::get_c_string_from_tagged(s_tagged);
    let trimmed = s.trim_start();
    
    let mut end = 0;
    let mut chars = trimmed.chars().peekable();
    if let Some(&c) = chars.peek() {
        if c == '+' || c == '-' {
            end += 1;
            chars.next();
        }
    }
    let mut has_dot = false;
    let mut has_e = false;
    while let Some(&c) = chars.peek() {
        if c.is_digit(10) {
            end += 1;
            chars.next();
        } else if c == '.' && !has_dot && !has_e {
            has_dot = true;
            end += 1;
            chars.next();
        } else if (c == 'e' || c == 'E') && !has_e {
            has_e = true;
            end += 1;
            chars.next();
            if let Some(&nc) = chars.peek() {
                if nc == '+' || nc == '-' {
                    end += 1;
                    chars.next();
                }
            }
        } else {
            break;
        }
    }
    
    let prefix = &trimmed[..end];
    if let Ok(val) = prefix.parse::<f64>() {
        crate::circ::box_number(val)
    } else {
        std::f64::NAN.to_bits()
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_isNaN(x: u64) -> u64 {
    let tag = x & TAG_MASK;
    if crate::dynamic_call::helpers::is_number_tag(tag) {
        let f = f64::from_bits(x);
        crate::circ::box_boolean(f.is_nan())
    } else if tag == TAG_STRING {
        let s = crate::get_c_string_from_tagged(x);
        if s.trim().parse::<f64>().is_err() {
            crate::circ::box_boolean(true)
        } else {
            crate::circ::box_boolean(false)
        }
    } else {
        crate::circ::box_boolean(true)
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_isFinite(x: u64) -> u64 {
    let tag = x & TAG_MASK;
    if crate::dynamic_call::helpers::is_number_tag(tag) {
        let f = f64::from_bits(x);
        crate::circ::box_boolean(f.is_finite())
    } else {
        crate::circ::box_boolean(false)
    }
}

