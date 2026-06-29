
use std::sync::Mutex;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use crate::circ::CircHeader;
use std::sync::atomic::Ordering;

struct FinalizerEntry {
    registry_tagged: u64,
    held_value: u64,
}

static FINALIZER_REGISTRY: Lazy<Mutex<HashMap<usize, Vec<FinalizerEntry>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

static FINALIZER_SENDER: Lazy<Mutex<Option<Sender<FinalizerEntry>>>> = Lazy::new(|| {
    Mutex::new(None)
});

#[no_mangle]
pub extern "C-unwind" fn __bs_finalizer_thread_init() {
    let mut sender_guard = FINALIZER_SENDER.lock().unwrap();
    if sender_guard.is_some() {
        return;
    }
    
    let (tx, rx): (Sender<FinalizerEntry>, Receiver<FinalizerEntry>) = channel();
    *sender_guard = Some(tx);
    
    thread::spawn(move || {
        while let Ok(entry) = rx.recv() {
            let mut pending = PENDING_FINALIZERS.lock().unwrap();
            pending.push(entry);
        }
    });
}

static PENDING_FINALIZERS: Lazy<Mutex<Vec<FinalizerEntry>>> = Lazy::new(|| {
    Mutex::new(Vec::new())
});

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_finalization_registry_register(registry_obj: *mut u8, target: u64, held_value: u64) {
    if target & 0xFFFF_0000_0000_0000 != 0xFFF6_0000_0000_0000 {
        return;
    }
    
    let target_ptr = (target & 0x0000_FFFF_FFFF_FFFF) as *mut u8;
    if target_ptr.is_null() {
        return;
    }
    
    let target_header = target_ptr.sub(CircHeader::SIZE) as *mut CircHeader;
    
    let mut old_flags = (*target_header).flags.load(Ordering::Relaxed);
    loop {
        let new_flags = old_flags | crate::circ::FINALIZER_TARGET;
        match (*target_header).flags.compare_exchange_weak(old_flags, new_flags, Ordering::AcqRel, Ordering::Relaxed) {
            Ok(_) => break,
            Err(e) => old_flags = e,
        }
    }
    
    let registry_tagged = (registry_obj as u64) | 0xFFF6_0000_0000_0000;
    crate::circ::circ_inc_tagged(registry_tagged);
    crate::circ::circ_inc_tagged(held_value);
    
    let mut registry = FINALIZER_REGISTRY.lock().unwrap();
    registry.entry(target_header as usize).or_default().push(FinalizerEntry {
        registry_tagged,
        held_value,
    });
}

pub unsafe fn enqueue_finalizers(target_header: *mut CircHeader) {
    let mut registry = FINALIZER_REGISTRY.lock().unwrap();
    if let Some(list) = registry.remove(&(target_header as usize)) {
        let sender_guard = FINALIZER_SENDER.lock().unwrap();
        if let Some(sender) = &*sender_guard {
            for entry in list {
                let _ = sender.send(entry);
            }
        } else {
            // Drop them
            for entry in list {
                crate::circ::circ_dec_tagged(entry.registry_tagged);
                crate::circ::circ_dec_tagged(entry.held_value);
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_drain_finalizers() {
    let mut pending = PENDING_FINALIZERS.lock().unwrap();
    for entry in pending.drain(..) {
        let registry_obj = (entry.registry_tagged & 0x0000_FFFF_FFFF_FFFF) as *mut u8;
        let closure_slot = registry_obj.add(16) as *mut u64;
        let closure_val = *closure_slot;
        
        if closure_val & 0xFFFF_0000_0000_0000 == 0xFFF9_0000_0000_0000 {
            let closure_ptr = (closure_val & 0x0000_FFFF_FFFF_FFFF) as *mut u8;
            let func_ptr_slot = closure_ptr as *mut extern "C-unwind" fn(u64, u64) -> u64;
            let func = *func_ptr_slot;
            func(closure_val, entry.held_value);
        }
        
        crate::circ::circ_dec_tagged(entry.registry_tagged);
        crate::circ::circ_dec_tagged(entry.held_value);
    }
}

pub unsafe extern "C-unwind" fn registry_register_method(env: u64, target: u64, held_value: u64) -> u64 {
    let closure_ptr = (env & 0x0000_FFFF_FFFF_FFFF) as *const u64;
    let obj_tagged = *(closure_ptr.add(3));
    let payload = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    __bs_finalization_registry_register(payload as *mut u8, target, held_value);
    0xFFF1_0000_0000_0000
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_FinalizationRegistry_new_1(callback: u64) -> u64 {
    let obj = crate::core::alloc::__bs_alloc_acyclic(&crate::core::vtable::FINALIZATION_REGISTRY_VTABLE, 24);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    let obj_ptr = payload as *mut u8;
    
    // offset 8 = inline props slot (zeroed by allocator)
    // offset 16 = callback closure
    let closure_slot = obj_ptr.add(16) as *mut u64;
    *closure_slot = callback;
    crate::circ::circ_inc_tagged(callback);

    let closure_tagged = crate::circ::create_builtin_method(obj, registry_register_method as *const u8);
    
    let props_slot = obj_ptr.add(8) as *mut *mut std::collections::HashMap<String, u64>;
    crate::objects::dynamic_props::set_inline_property_moved(props_slot, "register".to_string(), closure_tagged);
    
    obj
}
