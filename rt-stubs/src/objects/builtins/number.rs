use crate::gc;
const TAG_MASK: u64 = 0xFFFF_0000_0000_0000;

use crate::core::vtable::NUMBER_VTABLE;
use crate::core::alloc::__bs_alloc;
use crate::types::coercion::__bs_Number;
use crate::objects::dynamic_props::set_dynamic_property;

#[no_mangle]
pub unsafe extern "C" fn __bs_Number_new(val: u64) -> u64 {
    if (val & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 {
        __bs_Number_new_0()
    } else {
        __bs_Number_new_1(val)
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Number_new_0() -> u64 {
    let obj = __bs_alloc(&NUMBER_VTABLE, 8);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    set_dynamic_property(payload as *mut u8, "[[PrimitiveValue]]".to_string(), gc::box_number(0.0));
    obj
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Number_new_1(val: u64) -> u64 {
    let n_prim = __bs_Number(val);
    let obj = __bs_alloc(&NUMBER_VTABLE, 8);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    set_dynamic_property(payload as *mut u8, "[[PrimitiveValue]]".to_string(), n_prim);
    obj
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

#[no_mangle]
pub unsafe extern "C" fn __bs_number_isSafeInteger(x: u64) -> u64 {
    let tag = x & TAG_MASK;
    if tag == 0 || (tag > 0 && tag < 0xFFF0_0000_0000_0000) {
        let f = f64::from_bits(x);
        let max_safe = 9007199254740991.0;
        gc::box_boolean(f.is_finite() && f == f.trunc() && f.abs() <= max_safe)
    } else {
        gc::box_boolean(false)
    }
}
