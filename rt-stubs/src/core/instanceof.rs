use super::vtable::VTable;

/// Traverses the prototype chain of an object to verify if it inherits from a target shape ID.
///
/// Returns TAG_TRUE (0xFFF4_0000_0000_0000) if a match is found,
/// otherwise TAG_FALSE (0xFFF3_0000_0000_0000).
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_instanceof(obj_val: u64, target_shape_id: u64) -> u64 {
    // target_shape_id is passed as a NaN-boxed float, so we convert its bits back to f64 and cast to u64
    let target_shape_id_f64 = f64::from_bits(target_shape_id);
    let target_shape_id_u64 = target_shape_id_f64 as u64;

    // Check if the NaN tag is an object (Shared 0xFFF6, Owned 0x7FF6, Arena 0xFFFE)
    // The mask 0x7FF5 and target 0x7FF4 match exactly 0x7FF4 (True), 0x7FF6, 0x7FFC, and 0x7FFE.
    // TAG_TRUE will be rejected by the payload == 0 check below.
    let tag = obj_val & 0x7FF5_0000_0000_0000;
    if tag != 0x7FF4_0000_0000_0000 {
        return 0xFFF3_0000_0000_0000; // TAG_FALSE
    }
    // Extract the raw pointer (payload)
    let payload = obj_val & 0x0000_FFFF_FFFF_FFFF;
    if payload == 0 {
        return 0xFFF3_0000_0000_0000; // TAG_FALSE
    }
    let obj_ptr = payload as *const *const VTable;
    let mut vtable = *obj_ptr;

    let fmt = b"__bs_instanceof: obj=%lx target_shape=%lu\\n\\0".as_ptr() as *const libc::c_char;
    libc::printf(fmt, obj_val, target_shape_id_u64);
    libc::fflush(std::ptr::null_mut());

    // Traverse parent hierarchy in the prototype chain
    while !vtable.is_null() {
        let fmt2 = b"  vtable shape_id=%lu\n\0".as_ptr() as *const libc::c_char;
        libc::printf(fmt2, (*vtable).shape_id);
        libc::fflush(std::ptr::null_mut());
        
        if (*vtable).shape_id == target_shape_id_u64 {
            return 0xFFF4_0000_0000_0000; // TAG_TRUE
        }
        vtable = (*vtable).parent;
    }
    let fmt3 = b"  -> TAG_FALSE\n\0".as_ptr() as *const libc::c_char;
    libc::printf(fmt3);
    libc::fflush(std::ptr::null_mut());
    0xFFF3_0000_0000_0000 // TAG_FALSE
}
