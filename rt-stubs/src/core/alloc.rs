use crate::gc;
use super::vtable::VTable;

/// Allocates memory for a class object of size_in_bytes, zero-initializes it,
/// sets the vtable pointer at offset 0, and returns the tagged NaN-boxed pointer.
///
/// NaN-boxing tag for object: TAG_OBJECT = 0xFFF6_0000_0000_0000.
#[no_mangle]
pub unsafe extern "C" fn __bs_alloc(vtable_ptr: *const VTable, size_in_bytes: usize) -> u64 {
    // Objects have their first 8 bytes as the vtable.
    // The rest of the object may contain GC pointers. We'll conservatively scan everything.
    // Assuming size_in_bytes is a multiple of 8.
    let num_slots = (size_in_bytes / 8) as u16;
    let ptr = gc::gc_alloc(size_in_bytes, 0xFFF6, num_slots);
    
    // Store vtable pointer at offset 0
    let obj_ptr = ptr as *mut *const VTable;
    *obj_ptr = vtable_ptr;
    
    // Return as NaN-boxed Object pointer
    (ptr as u64) | 0xFFF6_0000_0000_0000
}

/// Allocates memory for a closure env of size_in_bytes, zero-initializes it,
/// and returns the tagged NaN-boxed closure pointer.
///
/// NaN-boxing tag for closure: TAG_CLOSURE = 0xFFF9_0000_0000_0000.
#[no_mangle]
pub unsafe extern "C" fn __bs_alloc_closure(size_in_bytes: usize) -> u64 {
    let num_slots = (size_in_bytes / 8) as u16;
    let ptr = gc::gc_alloc(size_in_bytes, 0xFFF9, num_slots);
    (ptr as u64) | 0xFFF9_0000_0000_0000
}

/// Allocates memory for a generator state struct of size_in_bytes, zero-initializes it,
/// and returns the tagged NaN-boxed generator pointer.
///
/// NaN-boxing tag for generator: TAG_GENERATOR = 0xFFFA_0000_0000_0000.
#[no_mangle]
pub unsafe extern "C" fn __bs_alloc_generator(size_in_bytes: usize) -> u64 {
    let num_slots = (size_in_bytes / 8) as u16;
    let ptr = gc::gc_alloc(size_in_bytes, 0xFFFA, num_slots);
    (ptr as u64) | 0xFFFA_0000_0000_0000
}
