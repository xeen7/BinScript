
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_WeakSet_new_0() -> u64 {
    let obj = super::set::__bs_Set_new_0();
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    let obj_ptr = payload as *mut u8;
    *(obj_ptr as *mut *const crate::core::vtable::VTable) = &crate::WEAKSET_VTABLE;
    obj
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_WeakSet_new_1(iterable: u64) -> u64 {
    let obj = super::set::__bs_Set_new_1(iterable);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    *(payload as *mut *const crate::core::vtable::VTable) = &crate::WEAKSET_VTABLE;
    obj
}
