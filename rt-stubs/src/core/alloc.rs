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
    println!("ALLOC_OBJECT: {:?}", vtable_ptr);
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
    
    // Initialize inline properties slot at offset 8 to null
    let props_slot = unsafe { obj_ptr.add(8) as *mut *mut std::collections::HashMap<String, u64> };
    *props_slot = std::ptr::null_mut();

    crate::verify::__bs_verify_track_alloc(obj_ptr);
    crate::circ::SHARED_ALLOCS.fetch_add(1, Ordering::Relaxed);

    // Return as NaN-boxed Object pointer (points to obj data, NOT header)
    {
        println!("Allocated Shared Object: {:p}", obj_ptr);
        (obj_ptr as u64) | 0xFFF6_0000_0000_0000
    }
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

    // Initialize inline properties slot at offset 8 to null
    let props_slot = obj_ptr.add(8) as *mut *mut std::collections::HashMap<String, u64>;
    *props_slot = std::ptr::null_mut();

    crate::verify::__bs_verify_track_alloc(obj_ptr);
    crate::circ::SHARED_ALLOCS.fetch_add(1, Ordering::Relaxed);

    // Return as NaN-boxed Object pointer (points to obj data, NOT header)
    {
        println!("Allocated Shared Object: {:p}", obj_ptr);
        (obj_ptr as u64) | 0xFFF6_0000_0000_0000
    }
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
    let total_size = CircHeader::SIZE + size_in_bytes;
    let (raw, alloc_size) = crate::slab::fast_alloc_shared(total_size);
    let header = raw as *mut CircHeader;
    
    (*header).local_rc = 0;
    (*header).global_rc.store(0, Ordering::Relaxed);
    (*header).owner_tid.store(crate::circ::NO_OWNER, Ordering::Relaxed);
    (*header).flags.store(crate::circ::VTABLE_PTR, Ordering::Relaxed);
    (*header).alloc_size = alloc_size;
    (*header).crc = 0;

    let obj_ptr = (raw as *mut u8).add(CircHeader::SIZE);

    // Store vtable pointer at offset 0 of the object
    let vtable_slot = obj_ptr as *mut *const VTable;
    *vtable_slot = vtable_ptr;

    // Initialize inline properties slot at offset 8 to null
    let props_slot = unsafe { obj_ptr.add(8) as *mut *mut std::collections::HashMap<String, u64> };
    *props_slot = std::ptr::null_mut();
    
    // Increment owned stats
    crate::circ::OWNED_ALLOCS.fetch_add(1, Ordering::Relaxed);

    #[cfg(feature = "debug_rc")]
    {
        println!("Allocated Owned Object: {:?}", obj_ptr);
    }

    crate::verify::__bs_verify_track_alloc(obj_ptr);
    

    // Return as NaN-boxed Owned Object pointer
    {
        println!("Allocated Owned Object: {:p}", obj_ptr);
        (obj_ptr as u64) | 0xFFFC_0000_0000_0000
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_free_owned(obj_ptr: *mut u8) {
    if obj_ptr.is_null() { return; } eprintln!("__bs_drop_owned: {:?}", obj_ptr);
    let header = obj_ptr.sub(CircHeader::SIZE) as *mut CircHeader;
    crate::circ::circ_destroy(header);
}

/// Allocates memory for a closure env of size_in_bytes, zero-initializes it,
/// and returns the tagged NaN-boxed closure pointer.
///
/// NaN-boxing tag for closure: TAG_CLOSURE = 0xFFF9_0000_0000_0000.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_alloc_closure(size_in_bytes: usize) -> u64 {
    let total_size = CircHeader::SIZE + size_in_bytes;

    let (raw, alloc_size) = crate::slab::fast_alloc_shared(total_size);

    let header = raw as *mut crate::circ::CircHeader;
    let tid = crate::circ::current_thread_id();
    let flags = crate::circ::IS_CLOSURE;
    (*header).local_rc = 1;
    (*header).global_rc.store(0, std::sync::atomic::Ordering::Relaxed);
    (*header).owner_tid.store(tid, std::sync::atomic::Ordering::Relaxed);
    (*header).flags.store(flags, std::sync::atomic::Ordering::Relaxed);
    (*header).alloc_size = alloc_size;
    (*header).crc = 0;

    let obj_ptr = (raw as *mut u8).add(crate::circ::CircHeader::SIZE);
    
    std::ptr::write_bytes(obj_ptr, 0, size_in_bytes);
    crate::verify::__bs_verify_track_alloc(obj_ptr);
    crate::circ::SHARED_ALLOCS.fetch_add(1, Ordering::Relaxed);
    println!("Allocated Shared Closure: {:p}", obj_ptr);
    {
        (obj_ptr as u64) | 0xFFF9_0000_0000_0000
    }
}

/// Allocates an owned closure in the heap. Returns a TAG_OWNED tagged pointer.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_alloc_owned_closure(size_in_bytes: usize) -> u64 {
    let total_size = CircHeader::SIZE + size_in_bytes;
    let (raw, alloc_size) = crate::slab::fast_alloc_shared(total_size);
    let header = raw as *mut CircHeader;
    
    (*header).local_rc = 0; // Owned objects don't use RC
    (*header).global_rc.store(0, Ordering::Relaxed);
    (*header).owner_tid.store(crate::circ::NO_OWNER, Ordering::Relaxed);
    (*header).flags.store(crate::circ::IS_CLOSURE, Ordering::Relaxed);
    (*header).alloc_size = alloc_size;
    (*header).crc = 0;

    let obj_ptr = (raw as *mut u8).add(CircHeader::SIZE);
    crate::verify::__bs_verify_track_alloc(obj_ptr);
    
    // Increment owned stats
    crate::circ::OWNED_ALLOCS.fetch_add(1, Ordering::Relaxed);
    
    (obj_ptr as u64) | 0x7FF9_0000_0000_0000 // TAG_OWNED_CLOSURE
}

/// Drops an owned object by destroying its contents and freeing the memory.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_drop_owned(obj_ptr: *mut u8) {
    if obj_ptr.is_null() { return; } eprintln!("__bs_drop_owned: {:?}", obj_ptr);
    
    let header = obj_ptr.sub(CircHeader::SIZE) as *mut CircHeader;
    
    // For Owned objects, we destroy and free them.
    crate::circ::circ_destroy(header);
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
    crate::circ::SHARED_ALLOCS.fetch_add(1, Ordering::Relaxed);
    println!("Allocated Shared Generator: {:p}", obj_ptr);
    (obj_ptr as u64) | 0xFFFA_0000_0000_0000
}
