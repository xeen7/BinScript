use crate::core::vtable::BOOLEAN_VTABLE;
use crate::core::alloc::__bs_alloc;
use crate::types::coercion::__bs_Boolean;
use crate::objects::dynamic_props::set_dynamic_property;

#[no_mangle]
pub unsafe extern "C" fn __bs_Boolean_new(val: u64) -> u64 {
    if (val & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 {
        __bs_Boolean_new_0()
    } else {
        __bs_Boolean_new_1(val)
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Boolean_new_0() -> u64 {
    let obj = __bs_alloc(&BOOLEAN_VTABLE, 8);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    set_dynamic_property(payload as *mut u8, "[[PrimitiveValue]]".to_string(), 0xFFF3_0000_0000_0000); // false
    obj
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Boolean_new_1(val: u64) -> u64 {
    let b_prim = __bs_Boolean(val);
    let obj = __bs_alloc(&BOOLEAN_VTABLE, 8);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    set_dynamic_property(payload as *mut u8, "[[PrimitiveValue]]".to_string(), b_prim);
    obj
}