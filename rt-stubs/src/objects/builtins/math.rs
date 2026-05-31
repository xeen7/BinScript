//! Math and Global helper function implementations for BinScript.

use crate::gc;


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

