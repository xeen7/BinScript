with open("rt-stubs/src/array/mod.rs", "r") as f:
    code = f.read()

new_funcs = """#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_alloc_owned_array() -> u64 {
    let raw = crate::slab::fast_alloc_shared(std::mem::size_of::<BsArray>()).0 as *mut BsArray;
    std::ptr::write(raw, BsArray {
        length: 0,
        capacity: 0,
        data: std::ptr::null_mut(),
    });
    let obj_ptr = raw as *mut u8;
    crate::verify::__bs_verify_track_alloc(obj_ptr);
    (obj_ptr as u64) | 0x7FFB_0000_0000_0000 // TAG_OWNED_ARRAY
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_alloc_arena_array(region_id: u32) -> u64 {
    let raw = crate::core::arena::alloc_in_arena(region_id, std::mem::size_of::<BsArray>()) as *mut BsArray;
    std::ptr::write(raw, BsArray {
        length: 0,
        capacity: 0,
        data: std::ptr::null_mut(),
    });
    (raw as u64) | 0x7FFA_0000_0000_0000 // TAG_ARENA_ARRAY
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_new() -> u64 {"""

code = code.replace("""#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_new() -> u64 {""", new_funcs)

code = code.replace("""pub(crate) unsafe fn untag_array(tagged: u64) -> *mut BsArray {
    let tag = tagged & TAG_MASK;
    if tag != TAG_ARRAY {
        return std::ptr::null_mut();
    }
    let payload = tagged & PAYLOAD_MASK;
    payload as *mut BsArray
}""", """pub(crate) unsafe fn untag_array(tagged: u64) -> *mut BsArray {
    let tag = tagged & TAG_MASK;
    if tag != TAG_ARRAY && tag != 0x7FFB_0000_0000_0000 && tag != 0x7FFA_0000_0000_0000 {
        return std::ptr::null_mut();
    }
    let payload = tagged & PAYLOAD_MASK;
    payload as *mut BsArray
}""")

with open("rt-stubs/src/array/mod.rs", "w") as f:
    f.write(code)
