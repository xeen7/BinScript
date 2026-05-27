//! Math and Global helper function implementations for BinScript.

use crate::gc;

const TAG_MASK: u64 = 0xFFFF_0000_0000_0000;
const TAG_STRING: u64 = 0xFFF7_0000_0000_0000;

// Simple, self-contained LCG RNG
static mut RNG_STATE: u64 = 123456789;

unsafe fn next_random() -> f64 {
    RNG_STATE = RNG_STATE.wrapping_mul(6364136223846793005).wrapping_add(1);
    ((RNG_STATE >> 12) as f64) / (1u64 << 52) as f64
}

// --- Math Functions ---

#[no_mangle]
pub unsafe extern "C" fn __bs_math_floor(x: u64) -> u64 {
    gc::box_number(f64::from_bits(x).floor())
}

#[no_mangle]
pub unsafe extern "C" fn __bs_math_ceil(x: u64) -> u64 {
    gc::box_number(f64::from_bits(x).ceil())
}

#[no_mangle]
pub unsafe extern "C" fn __bs_math_round(x: u64) -> u64 {
    gc::box_number(f64::from_bits(x).round())
}

#[no_mangle]
pub unsafe extern "C" fn __bs_math_abs(x: u64) -> u64 {
    gc::box_number(f64::from_bits(x).abs())
}

#[no_mangle]
pub unsafe extern "C" fn __bs_math_sqrt(x: u64) -> u64 {
    gc::box_number(f64::from_bits(x).sqrt())
}

#[no_mangle]
pub unsafe extern "C" fn __bs_math_pow(x: u64, y: u64) -> u64 {
    gc::box_number(f64::from_bits(x).powf(f64::from_bits(y)))
}

#[no_mangle]
pub unsafe extern "C" fn __bs_math_min(x: u64, y: u64) -> u64 {
    gc::box_number(f64::from_bits(x).min(f64::from_bits(y)))
}

#[no_mangle]
pub unsafe extern "C" fn __bs_math_max(x: u64, y: u64) -> u64 {
    gc::box_number(f64::from_bits(x).max(f64::from_bits(y)))
}

#[no_mangle]
pub unsafe extern "C" fn __bs_math_log(x: u64) -> u64 {
    gc::box_number(f64::from_bits(x).ln())
}

#[no_mangle]
pub unsafe extern "C" fn __bs_math_log2(x: u64) -> u64 {
    gc::box_number(f64::from_bits(x).log2())
}

#[no_mangle]
pub unsafe extern "C" fn __bs_math_sin(x: u64) -> u64 {
    gc::box_number(f64::from_bits(x).sin())
}

#[no_mangle]
pub unsafe extern "C" fn __bs_math_cos(x: u64) -> u64 {
    gc::box_number(f64::from_bits(x).cos())
}

#[no_mangle]
pub unsafe extern "C" fn __bs_math_tan(x: u64) -> u64 {
    gc::box_number(f64::from_bits(x).tan())
}

#[no_mangle]
pub unsafe extern "C" fn __bs_math_random() -> u64 {
    gc::box_number(next_random())
}

#[no_mangle]
pub unsafe extern "C" fn __bs_math_trunc(x: u64) -> u64 {
    gc::box_number(f64::from_bits(x).trunc())
}

// --- Global Functions ---

#[no_mangle]
pub unsafe extern "C" fn __bs_parseInt_1(s_tagged: u64) -> u64 {
    let s = crate::get_c_string_from_tagged(s_tagged);
    let trimmed = s.trim();
    if let Ok(val) = i64::from_str_radix(trimmed, 10) {
        gc::box_number(val as f64)
    } else {
        if let Ok(val) = trimmed.parse::<f64>() {
            gc::box_number(val.trunc())
        } else {
            std::f64::NAN.to_bits()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_parseInt_2(s_tagged: u64, radix_tagged: u64) -> u64 {
    let s = crate::get_c_string_from_tagged(s_tagged);
    let radix = f64::from_bits(radix_tagged) as u32;
    if radix < 2 || radix > 36 {
        return std::f64::NAN.to_bits();
    }
    let trimmed = s.trim();
    if let Ok(val) = i64::from_str_radix(trimmed, radix) {
        gc::box_number(val as f64)
    } else {
        std::f64::NAN.to_bits()
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_parseFloat(s_tagged: u64) -> u64 {
    let s = crate::get_c_string_from_tagged(s_tagged);
    if let Ok(val) = s.trim().parse::<f64>() {
        gc::box_number(val)
    } else {
        std::f64::NAN.to_bits()
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_isNaN(x: u64) -> u64 {
    let tag = x & TAG_MASK;
    if tag == 0 || (tag > 0 && tag < 0xFFF0_0000_0000_0000) {
        let f = f64::from_bits(x);
        gc::box_boolean(f.is_nan())
    } else if tag == TAG_STRING {
        let s = crate::get_c_string_from_tagged(x);
        if s.trim().parse::<f64>().is_err() {
            gc::box_boolean(true)
        } else {
            gc::box_boolean(false)
        }
    } else {
        gc::box_boolean(true)
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_isFinite(x: u64) -> u64 {
    let tag = x & TAG_MASK;
    if tag == 0 || (tag > 0 && tag < 0xFFF0_0000_0000_0000) {
        let f = f64::from_bits(x);
        gc::box_boolean(f.is_finite())
    } else {
        gc::box_boolean(false)
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_number_isInteger(x: u64) -> u64 {
    let tag = x & TAG_MASK;
    if tag == 0 || (tag > 0 && tag < 0xFFF0_0000_0000_0000) {
        let f = f64::from_bits(x);
        gc::box_boolean(f.is_finite() && f == f.trunc())
    } else {
        gc::box_boolean(false)
    }
}
