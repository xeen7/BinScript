use crate::circ::CircHeader;
use super::vtable::VTable;
use std::sync::atomic::Ordering;

/// Allocates memory for a class object of size_in_bytes, zero-initializes it,
/// sets the vtable pointer at offset 0, and returns the tagged NaN-boxed pointer.
///
/// Layout: `[CircHeader (8 bytes)] [vtable ptr | field_0 | field_1 | ...]`
///
/// NaN-boxing tag for object: TAG_OBJECT = 0xFFF6_0000_0000_0000.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_alloc(vtable_ptr: *const VTable, size_in_bytes: usize) -> u64 {
    let total_size = CircHeader::SIZE + size_in_bytes;

    let (raw, alloc_size) = crate::slab::fast_alloc_shared(total_size);

    // Init CircHeader at allocation base
    let header = raw as *mut CircHeader;
    (*header).local_rc = 1;
    (*header).global_rc.store(0, Ordering::Relaxed);
    (*header).owner_tid.store(crate::circ::current_thread_id(), Ordering::Relaxed);
    (*header).flags.store(crate::circ::VTABLE_PTR, Ordering::Relaxed);
    (*header).alloc_size = alloc_size;
    (*header).crc = 0;

    // Object data starts after CircHeader
    let obj_ptr = (raw as *mut u8).add(CircHeader::SIZE);

    // Store vtable pointer at offset 0 of the object
    let vtable_slot = obj_ptr as *mut *const VTable;
    *vtable_slot = vtable_ptr;
    
    crate::verify::__bs_verify_track_alloc(obj_ptr);

    // Return as NaN-boxed Object pointer (points to obj data, NOT header)
    (obj_ptr as u64) | 0xFFF6_0000_0000_0000
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_alloc_acyclic(vtable_ptr: *const VTable, size_in_bytes: usize) -> u64 {
    let total_size = CircHeader::SIZE + size_in_bytes;

    let (raw, alloc_size) = crate::slab::fast_alloc_shared(total_size);

    // Init CircHeader at allocation base
    let header = raw as *mut CircHeader;
    (*header).local_rc = 1;
    (*header).global_rc.store(0, Ordering::Relaxed);
    (*header).owner_tid.store(crate::circ::current_thread_id(), Ordering::Relaxed);
    (*header).flags.store(crate::circ::VTABLE_PTR | crate::circ::ACYCLIC, Ordering::Relaxed);
    (*header).alloc_size = alloc_size;
    (*header).crc = 0;

    // Object data starts after CircHeader
    let obj_ptr = (raw as *mut u8).add(CircHeader::SIZE);

    // Store vtable pointer at offset 0 of the object
    let vtable_slot = obj_ptr as *mut *const VTable;
    *vtable_slot = vtable_ptr;

    crate::verify::__bs_verify_track_alloc(obj_ptr);

    // Return as NaN-boxed Object pointer (points to obj data, NOT header)
    (obj_ptr as u64) | 0xFFF6_0000_0000_0000
}

/// Allocates memory for a class object of size_in_bytes, zero-initializes it,
/// sets the vtable pointer at offset 0, and returns the tagged NaN-boxed pointer.
/// This allocation is Owned and does NOT include a `CircHeader`.
///
/// Layout: `[vtable ptr | field_0 | field_1 | ...]`
///
/// NaN-boxing tag for owned object: TAG_OWNED_OBJECT = 0xFFFC_0000_0000_0000.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_alloc_owned(vtable_ptr: *const VTable, size_in_bytes: usize) -> u64 {
    // We use the same fast allocator as Shared objects, but we don't prepend a CircHeader
    let (raw, _alloc_size) = crate::slab::fast_alloc_shared(size_in_bytes);

    let obj_ptr = raw as *mut u8;

    // Store vtable pointer at offset 0 of the object
    let vtable_slot = obj_ptr as *mut *const VTable;
    *vtable_slot = vtable_ptr;

    crate::verify::__bs_verify_track_alloc(obj_ptr);

    // Return as NaN-boxed Owned Object pointer
    (obj_ptr as u64) | 0xFFFC_0000_0000_0000
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_free_owned(obj_ptr: *mut u8) {
    if obj_ptr.is_null() { return; }
    
    let vtable_ptr = *(obj_ptr as *mut *const VTable);
    if vtable_ptr.is_null() { return; }
    
    let fields_count = (*vtable_ptr).fields_count as usize;
    let size_in_bytes = 8 * (2 + fields_count);
    
    let alloc_size = if size_in_bytes <= 32 { 32 } 
                     else if size_in_bytes <= 64 { 64 } 
                     else if size_in_bytes <= 128 { 128 } 
                     else if size_in_bytes <= 256 { 256 } 
                     else { 0 };
                     
    crate::slab::fast_free_shared(obj_ptr, alloc_size);
}

/// Allocates memory for a closure env of size_in_bytes, zero-initializes it,
/// and returns the tagged NaN-boxed closure pointer.
///
/// NaN-boxing tag for closure: TAG_CLOSURE = 0xFFF9_0000_0000_0000.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_alloc_closure(size_in_bytes: usize) -> u64 {
    let total_size = CircHeader::SIZE + size_in_bytes;

    let (raw, alloc_size) = crate::slab::fast_alloc_shared(total_size);

    let header = raw as *mut CircHeader;
    (*header).local_rc = 1;
    (*header).global_rc.store(0, Ordering::Relaxed);
    (*header).owner_tid.store(crate::circ::current_thread_id(), Ordering::Relaxed);
    (*header).flags.store(crate::circ::IS_CLOSURE, Ordering::Relaxed);
    (*header).alloc_size = alloc_size;
    (*header).crc = 0;

    let obj_ptr = (raw as *mut u8).add(CircHeader::SIZE);
    std::ptr::write_bytes(obj_ptr, 0, size_in_bytes);
    crate::verify::__bs_verify_track_alloc(obj_ptr);
    (obj_ptr as u64) | 0xFFF9_0000_0000_0000
}

/// Allocates memory for a generator state struct of size_in_bytes, zero-initializes it,
/// and returns the tagged NaN-boxed generator pointer.
///
/// NaN-boxing tag for generator: TAG_GENERATOR = 0xFFFA_0000_0000_0000.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_alloc_generator(size_in_bytes: usize) -> u64 {
    let total_size = CircHeader::SIZE + size_in_bytes;

    let (raw, alloc_size) = crate::slab::fast_alloc_shared(total_size);

    let header = raw as *mut CircHeader;
    (*header).local_rc = 1;
    (*header).global_rc.store(0, Ordering::Relaxed);
    (*header).owner_tid.store(crate::circ::current_thread_id(), Ordering::Relaxed);
    (*header).flags.store(crate::circ::IS_GENERATOR, Ordering::Relaxed);
    (*header).alloc_size = alloc_size;
    (*header).crc = 0;

    let obj_ptr = (raw as *mut u8).add(CircHeader::SIZE);
    std::ptr::write_bytes(obj_ptr, 0, size_in_bytes);
    crate::verify::__bs_verify_track_alloc(obj_ptr);
    // println!("__bs_alloc_generator: {:?}", obj_ptr);
    (obj_ptr as u64) | 0xFFFA_0000_0000_0000
}
