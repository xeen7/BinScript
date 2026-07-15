
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_WeakMap_new_0() -> u64 {
    crate::core::vtable::__bs_init_weakmap_prototype();
    let obj = super::map::__bs_Map_new_0();
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    let obj_ptr = payload as *mut u8;
    *(obj_ptr as *mut *const crate::core::vtable::VTable) = &crate::WEAKMAP_VTABLE;
    crate::objects::dynamic_props::set_dynamic_property(payload as *mut u8, "__proto__".to_string(), crate::core::vtable::WEAKMAP_PROTOTYPE);
    obj
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_WeakMap_new_1(iterable: u64) -> u64 {
    let obj = super::map::__bs_Map_new_1(iterable);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    *(payload as *mut *const crate::core::vtable::VTable) = &crate::WEAKMAP_VTABLE;
    obj
}
