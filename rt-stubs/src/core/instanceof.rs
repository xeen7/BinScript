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

    // Check if the NaN tag is exactly TAG_OBJECT (0xFFF6)
    let tag = obj_val & 0xFFFF_0000_0000_0000;
    if tag != 0xFFF6_0000_0000_0000 {
        return 0xFFF3_0000_0000_0000; // TAG_FALSE
    }
    // Extract the raw pointer (payload)
    let payload = obj_val & 0x0000_FFFF_FFFF_FFFF;
    if payload == 0 {
        return 0xFFF3_0000_0000_0000; // TAG_FALSE
    }
    let obj_ptr = payload as *const *const VTable;
    let mut vtable = *obj_ptr;
    // Traverse parent hierarchy in the prototype chain
    while !vtable.is_null() {
        if (*vtable).shape_id == target_shape_id_u64 {
            return 0xFFF4_0000_0000_0000; // TAG_TRUE
        }
        vtable = (*vtable).parent;
    }
    0xFFF3_0000_0000_0000 // TAG_FALSE
}
