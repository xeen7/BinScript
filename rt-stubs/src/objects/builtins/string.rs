use crate::core::vtable::STRING_VTABLE;
use crate::core::alloc::__bs_alloc;
use crate::types::coercion::__bs_String;
use crate::types::string_utils::{get_c_string_from_tagged, create_tagged_string};
use crate::objects::dynamic_props::set_dynamic_property;

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_String_new(val: u64) -> u64 {
    if (val & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 {
        __bs_String_new_0()
    } else {
        __bs_String_new_1(val)
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_String_new_0() -> u64 {
    let obj = __bs_alloc(&STRING_VTABLE, 16);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    let s_prim = create_tagged_string("");
    set_dynamic_property(payload as *mut u8, "[[PrimitiveValue]]".to_string(), s_prim);
    set_dynamic_property(payload as *mut u8, "length".to_string(), crate::circ::box_number(0.0));
    obj
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_String_new_1(val: u64) -> u64 {
    let s_prim = __bs_String(val);
    let obj = __bs_alloc(&STRING_VTABLE, 16);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    set_dynamic_property(payload as *mut u8, "[[PrimitiveValue]]".to_string(), s_prim);
    
    let s_str = get_c_string_from_tagged(s_prim);
    set_dynamic_property(payload as *mut u8, "length".to_string(), crate::circ::box_number(s_str.len() as f64));
    
    let chars: Vec<char> = s_str.chars().collect();
    for (i, ch) in chars.iter().enumerate() {
        let ch_str = ch.to_string();
        let ch_tagged = create_tagged_string(&ch_str);
        set_dynamic_property(payload as *mut u8, i.to_string(), ch_tagged);
    }
    obj
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_string_fromCharCode(code: u64) -> u64 {
    let f = f64::from_bits(code);
    let ch = (f as u32).try_into().unwrap_or('\0');
    let ch_str = ch.to_string();
    create_tagged_string(&ch_str)
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_string_fromCodePoint(code: u64) -> u64 {
    __bs_string_fromCharCode(code)
}