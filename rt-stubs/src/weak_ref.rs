use std::sync::Mutex;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use crate::circ::CircHeader;
use std::sync::atomic::Ordering;

/// Global registry mapping a target object's header pointer to a list of WeakRef object pointers.
static WEAK_REGISTRY: Lazy<Mutex<HashMap<usize, Vec<usize>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

/// Creates a new WeakRef target link.
/// `weakref_obj` is the pointer to the WeakRef object data (after its header).
/// `target` is the NaN-boxed object pointer.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_weakref_new(weakref_obj: *mut u8, target: u64) {
    if target & 0xFFFF_0000_0000_0000 != 0xFFF6_0000_0000_0000 {
        // Not a reference object, WeakRef to primitive is undefined behavior or not allowed in JS,
        // but we'll just ignore it or it's a TypeError in JS.
        return;
    }
    
    let target_ptr = (target & 0x0000_FFFF_FFFF_FFFF) as *mut u8;
    if target_ptr.is_null() {
        return;
    }
    
    let target_header = target_ptr.sub(CircHeader::SIZE) as *mut CircHeader;
    
    // Set WEAKREF_TARGET flag on target
    let mut old_flags = (*target_header).flags.load(Ordering::Relaxed);
    loop {
        let new_flags = old_flags | crate::circ::WEAKREF_TARGET;
        match (*target_header).flags.compare_exchange_weak(old_flags, new_flags, Ordering::AcqRel, Ordering::Relaxed) {
            Ok(_) => break,
            Err(e) => old_flags = e,
        }
    }
    
    // Register the weakref
    let mut registry = WEAK_REGISTRY.lock().unwrap();
    registry.entry(target_header as usize).or_default().push(weakref_obj as usize);
    
    // Store the target raw pointer in the WeakRef object's offset 16
    let target_slot = weakref_obj.add(16) as *mut u64;
    *target_slot = target;
}

/// Dereferences a WeakRef.
/// `weakref_obj` is the pointer to the WeakRef object data.
/// Returns the NaN-boxed target object, or undefined (0xFFF1_0000_0000_0000) if collected.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_weakref_deref(weakref_obj: *mut u8) -> u64 {
    let target_slot = weakref_obj.add(16) as *mut u64;
    let target = *target_slot;
    
    if target == 0 || target == 0xFFF1_0000_0000_0000 {
        return 0xFFF1_0000_0000_0000; // undefined
    }
    
    // Increment RC to give the caller a strong reference
    let target_ptr = (target & 0x0000_FFFF_FFFF_FFFF) as *mut u8;
    let target_header = target_ptr.sub(CircHeader::SIZE) as *mut CircHeader;
    crate::circ::circ_inc(target_header);
    
    target
}

/// Called when a WeakRef object itself is destroyed.
/// Removes it from the registry so we don't hold dangling pointers.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_weakref_drop(weakref_obj: *mut u8) {
    let target_slot = weakref_obj.add(16) as *mut u64;
    let target = *target_slot;
    if target != 0 && target != 0xFFF1_0000_0000_0000 {
        let target_ptr = (target & 0x0000_FFFF_FFFF_FFFF) as *mut u8;
        let target_header = target_ptr.sub(CircHeader::SIZE) as *mut CircHeader;
        
        let mut registry = WEAK_REGISTRY.lock().unwrap();
        if let Some(list) = registry.get_mut(&(target_header as usize)) {
            list.retain(|&x| x != (weakref_obj as usize));
            if list.is_empty() {
                registry.remove(&(target_header as usize));
            }
        }
    }
}

/// Called by `circ_destroy` when an object with WEAKREF_TARGET is destroyed.
/// Nullifies all WeakRefs pointing to it.
pub unsafe fn nullify_weak_refs(target_header: *mut CircHeader) {
    let mut registry = WEAK_REGISTRY.lock().unwrap();
    if let Some(list) = registry.remove(&(target_header as usize)) {
        for weakref_obj in list {
            let target_slot = (weakref_obj as *mut u8).add(16) as *mut u64;
            *target_slot = 0xFFF1_0000_0000_0000; // undefined
        }
    }
}

pub unsafe extern "C-unwind" fn weakref_deref_method(env: u64) -> u64 {
    let closure_ptr = (env & 0x0000_FFFF_FFFF_FFFF) as *const u64;
    let obj_tagged = *(closure_ptr.add(3));
    let payload = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    __bs_weakref_deref(payload as *mut u8)
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_WeakRef_new_1(target: u64) -> u64 {
    let obj = crate::core::alloc::__bs_alloc_acyclic(&crate::core::vtable::WEAKREF_VTABLE, 24);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    let obj_ptr = payload as *mut u8;
    
    __bs_weakref_new(obj_ptr, target);

    // Create dynamic method 'deref'
    let closure_tagged = crate::circ::create_builtin_method(obj, weakref_deref_method as *const u8);
    
    let props_slot = obj_ptr.add(8) as *mut *mut std::collections::HashMap<String, u64>;
    crate::objects::dynamic_props::set_inline_property_moved(props_slot, "deref".to_string(), closure_tagged);
    
    obj
}
