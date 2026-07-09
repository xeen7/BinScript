use std::sync::Mutex;
use std::collections::HashSet;
use once_cell::sync::Lazy;

use std::sync::atomic::{AtomicBool, Ordering};

pub static VERIFY_MEMORY_ENABLED: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub extern "C-unwind" fn __bs_set_verify_memory(enabled: bool) {
    VERIFY_MEMORY_ENABLED.store(enabled, Ordering::SeqCst);
}

static TRACKED_ALLOCS: Lazy<Mutex<HashSet<usize>>> = Lazy::new(|| {
    Mutex::new(HashSet::new())
});

static FREED_ALLOCS: Lazy<Mutex<HashSet<usize>>> = Lazy::new(|| {
    Mutex::new(HashSet::new())
});

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_verify_track_alloc(ptr: *mut u8) {
    if !VERIFY_MEMORY_ENABLED.load(Ordering::Relaxed) || ptr.is_null() { return; }
    let mut tracked = TRACKED_ALLOCS.lock().unwrap();
    tracked.insert(ptr as usize);
    
    let mut freed = FREED_ALLOCS.lock().unwrap();
    freed.remove(&(ptr as usize));
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_verify_track_free(ptr: *mut u8) {
    if !VERIFY_MEMORY_ENABLED.load(Ordering::Relaxed) || ptr.is_null() { return; }
    let mut tracked = TRACKED_ALLOCS.lock().unwrap();
    tracked.remove(&(ptr as usize));
    
    let mut freed = FREED_ALLOCS.lock().unwrap();
    freed.insert(ptr as usize);
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __verify_load(ptr: *mut u8) {
    if !VERIFY_MEMORY_ENABLED.load(Ordering::Relaxed) || ptr.is_null() { return; }
    verify_load_inner(ptr);
}

/// Rust-ABI inner function so that panics can unwind normally (needed for #[should_panic] tests).
pub unsafe fn verify_load_inner(ptr: *mut u8) {
    let freed = FREED_ALLOCS.lock().unwrap();
    if freed.contains(&(ptr as usize)) {
        // Drop the lock before panicking to avoid poisoning issues
        drop(freed);
        panic!("FATAL ERROR: Use-After-Free detected at pointer {:?}", ptr);
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __verify_store(ptr: *mut u8) {
    if !VERIFY_MEMORY_ENABLED.load(Ordering::Relaxed) || ptr.is_null() { return; }
    
    let freed = FREED_ALLOCS.lock().unwrap();
    if freed.contains(&(ptr as usize)) {
        panic!("FATAL ERROR: Write-After-Free detected at pointer {:?}", ptr);
    }
}

extern "C-unwind" {
    fn __bs_cycle_buffer_flush_local();
    fn __bs_cycle_collector_flush();
    fn __bs_cleanup_global_this();
    fn __bs_cleanup_dynamic_properties();
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_verify_check_leaks() {
    if !VERIFY_MEMORY_ENABLED.load(Ordering::Relaxed) { return; }
    
    // Clean up statically allocated singletons before leak check
    __bs_cleanup_global_this();
    __bs_cleanup_dynamic_properties();

    // Flush the local cycle buffer into the cycle collector, then force a final collection
    __bs_cycle_buffer_flush_local();
    __bs_cycle_collector_flush();
    
    let tracked = TRACKED_ALLOCS.lock().unwrap();
    if !tracked.is_empty() {
        crate::circ::__bs_print_rc_stats();
        println!("FATAL ERROR: Memory Leak detected! {} allocations not freed", tracked.len());
        for &actual_ptr in tracked.iter() {
            let header = unsafe { (actual_ptr as *mut u8).sub(crate::circ::CircHeader::SIZE) as *mut crate::circ::CircHeader };
            let size = unsafe { (*header).alloc_size };
            let color = unsafe { (*header).get_color() };
            let flags = unsafe { (*header).flags.load(std::sync::atomic::Ordering::Relaxed) };
            let local_rc = unsafe { (*header).local_rc };
            let global_rc = unsafe { (*header).global_rc.load(std::sync::atomic::Ordering::Relaxed) };
            let vtable_ptr = unsafe { *(actual_ptr as *const u64) };
            let field_1 = unsafe { *(actual_ptr as *const u64).add(1) };
            println!("Leaked pointer: 0x{:x}, size: {}, color: {}, flags: 0x{:x}, local_rc: {}, global_rc: {}", actual_ptr, size, color, flags, local_rc, global_rc);
            println!("  Content: [0x{:x}, 0x{:x}]", vtable_ptr, field_1);
            if (flags & crate::circ::VTABLE_PTR) != 0 {
                let vtable_ptr_ptr = actual_ptr as *const *const crate::core::vtable::VTable;
                let vtable = unsafe { *vtable_ptr_ptr };
                if !vtable.is_null() {
                    let name = unsafe { std::ffi::CStr::from_ptr((*vtable).name as *const libc::c_char).to_string_lossy().into_owned() };
                    println!("Leaked object of class: {}", name);
                }
            }
        }
        std::process::exit(1);
    }
}
