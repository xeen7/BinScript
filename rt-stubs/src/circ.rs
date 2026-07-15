//! CIRC — Concurrent Immediate Reference Counting.
//!
//! Base implementation: single atomic `rc` field. BiRC (biased local/global split)
//! is layered on top in Phase 6.
//!
//! Hard constraints (from memory_model_final.md §31):
//! - `circ_inc` uses `Relaxed` — caller already holds a reference.
//! - `circ_dec` uses `AcqRel` — **never** `Relaxed`.
//! - `drop_fn` must **never** call `free(self)`.
//! - `CircHeader` is always **prepended** to the object.

use std::sync::atomic::{AtomicU32, AtomicI32, Ordering};
use std::cell::Cell;

// ── Thread ID management ───────────────────────────────────────────────────

static NEXT_TID: AtomicU32 = AtomicU32::new(1); // 0 is NO_OWNER

thread_local! {
    static THREAD_ID: Cell<u32> = Cell::new(0);
}



#[no_mangle]
pub extern "C-unwind" fn current_thread_id() -> u32 {
    THREAD_ID.with(|id| {
        let mut cur = id.get();
        if cur == 0 {
            cur = NEXT_TID.fetch_add(1, Ordering::Relaxed);
            id.set(cur);
        }
        cur
    })
}



pub const NO_OWNER: u32 = 0;

// ── CircHeader flags ───────────────────────────────────────────────────────

pub const ACYCLIC: u16 = 1 << 0;
pub const IN_NURSERY: u16 = 1 << 1;
pub const FORWARDED: u16 = 1 << 2;
pub const ZOMBIE: u16 = 1 << 3;
pub const VTABLE_PTR: u16 = 1 << 4;
pub const WEAKREF_TARGET: u16 = 1 << 5;
pub const FINALIZER_TARGET: u16 = 1 << 6;
pub const IS_CLOSURE: u16 = 1 << 10;
pub const IS_ARRAY: u16 = 1 << 11;
pub const IS_GENERATOR: u16 = 1 << 12;

// Color bits (3 bits) for Bacon-Rajan cycle collection
pub const COLOR_MASK: u16 = 0b111 << 7;
pub const COLOR_BLACK: u16 = 0 << 7;
pub const COLOR_GRAY: u16 = 1 << 7;
pub const COLOR_WHITE: u16 = 2 << 7;
pub const COLOR_PURPLE: u16 = 3 << 7;
pub const COLOR_FREEING: u16 = 4 << 7;

// ── CircHeader layout ──────────────────────────────────────────────────────

#[repr(C, align(8))]
pub struct CircHeader {
    pub local_rc: u32,
    pub global_rc: AtomicI32,
    pub owner_tid: AtomicU32,
    pub flags: std::sync::atomic::AtomicU16,
    pub alloc_size: u16,
    pub crc: u32, // Cycle Reference Count (used exclusively by cycle collector)
}

impl CircHeader {
    pub const SIZE: usize = std::mem::size_of::<CircHeader>();
}

// ── CIRC ABI (BiRC) ────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C-unwind" fn circ_inc(header: *mut CircHeader) {
    if header.is_null() {
        return;
    }
    ACTUAL_RC_INCS.fetch_add(1, Ordering::Relaxed);
    let obj_ptr = (header as *mut u8).add(CircHeader::SIZE);
    let global_val = (*header).global_rc.load(Ordering::Relaxed);
    #[cfg(feature = "debug_rc")]
    eprintln!("circ_inc: {:?} (local_rc before: {}, global_rc: {})", obj_ptr, (*header).local_rc, global_val);
    let cur_tid = current_thread_id();
    let owner = (*header).owner_tid.load(Ordering::Relaxed);

    if owner == cur_tid {
        // Fast path: owning thread. No other thread can be accessing this.
        (*header).local_rc += 1;
    } else {
        // Slow path: shared across threads (owner_tid == NO_OWNER)
        (*header).global_rc.fetch_add(1, Ordering::SeqCst);
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn circ_dec(header_ptr: *mut CircHeader) {
    if header_ptr.is_null() {
        return;
    }
    ACTUAL_RC_DECS.fetch_add(1, Ordering::Relaxed);
    let header = &*header_ptr;
    let obj_ptr = (header_ptr as *mut u8).add(CircHeader::SIZE);
    let global_val = (*header).global_rc.load(Ordering::Relaxed);
    #[cfg(feature = "debug_rc")]
    eprintln!("circ_dec: {:?} (local_rc before: {}, global_rc: {})", obj_ptr, header.local_rc, global_val);

    
    let tid = current_thread_id();
    let owner = header.owner_tid.load(Ordering::Relaxed);

    if owner == tid {
        // BiRC local fast path
        if header.local_rc > 0 {
            (*header_ptr).local_rc -= 1;
            let color = header.get_color();
            if color == crate::circ::COLOR_WHITE {
                header.set_color(crate::circ::COLOR_BLACK);
                SHARED_FREES.fetch_add(1, Ordering::Relaxed);
                // eprintln!("circ_dec (local, >0): freeing white");
                circ_destroy(header_ptr);
                return;
            }
            if (*header_ptr).local_rc == 0 && header.global_rc.load(Ordering::Acquire) <= 0 {
                if color == crate::circ::COLOR_PURPLE {
                    // eprintln!("circ_dec (local, >0): ignoring purple");
                    return;
                }
                // eprintln!("circ_dec (local, >0): freeing");
                SHARED_FREES.fetch_add(1, Ordering::Relaxed);
                circ_destroy(header_ptr);
                return;
            }
        } else {
            let prev = header.global_rc.fetch_sub(1, Ordering::AcqRel);
            let color = header.get_color();
            if color == crate::circ::COLOR_WHITE {
                header.set_color(crate::circ::COLOR_BLACK);
                SHARED_FREES.fetch_add(1, Ordering::Relaxed);
                // eprintln!("circ_dec (local, 0): freeing white");
                circ_destroy(header_ptr);
                return;
            }
            if prev == 1 {
                if color == crate::circ::COLOR_PURPLE {
                    // eprintln!("circ_dec (local, 0): ignoring purple");
                    return;
                }
                // eprintln!("circ_dec (local, 0): freeing");
                SHARED_FREES.fetch_add(1, Ordering::Relaxed);
                circ_destroy(header_ptr);
                return;
            }
        }
        
        let flags = header.flags.load(Ordering::Relaxed);
        if flags & ACYCLIC == 0 {
            // Buffer into cycle collector if not acyclic and RC > 0
            crate::cycle_buffer::__bs_cycle_buffer_push(header_ptr);
        }
    } else {
        // Global RC path
        let prev = header.global_rc.fetch_sub(1, Ordering::AcqRel);
        let current_global = header.global_rc.load(Ordering::Relaxed);
        let total_rc = header.local_rc as i64 + current_global as i64;
        
        let color = header.get_color();
        // eprintln!("circ_dec (global): {:?}, local_rc={}, prev_global={}, total_rc={}, color={}", header_ptr, header.local_rc, prev, total_rc, color);

        if color == crate::circ::COLOR_WHITE {
            header.set_color(crate::circ::COLOR_BLACK);
            SHARED_FREES.fetch_add(1, Ordering::Relaxed);
            // eprintln!("circ_dec (global): freeing white");
            circ_destroy(header_ptr);
            return;
        }
        
        if total_rc <= 0 {
            if color == crate::circ::COLOR_PURPLE {
                // eprintln!("circ_dec (global): ignoring purple");
                return;
            }
            // eprintln!("circ_dec (global): freeing <= 0");
            SHARED_FREES.fetch_add(1, Ordering::Relaxed);
            circ_destroy(header_ptr);
            return;
        }
    }    
    let flags = header.flags.load(Ordering::Relaxed);
    if flags & ACYCLIC == 0 {
        crate::cycle_buffer::__bs_cycle_buffer_push(header_ptr);
    }
}

/// Promotes a local object to a globally shared object.
/// Must be called by the owning thread before the object reference 
/// is made visible to other threads (e.g., stored in a shared field).
#[no_mangle]
pub unsafe extern "C-unwind" fn circ_promote(header: *mut CircHeader) {
    if header.is_null() {
        return;
    }
    let cur_tid = current_thread_id();
    let owner = (*header).owner_tid.load(Ordering::Relaxed);
    
    if owner == cur_tid {
        // Flush local_rc
        let local = (*header).local_rc;
        (*header).local_rc = 0;
        (*header).global_rc.fetch_add(local as i32, Ordering::Relaxed);
        
        // Release memory ordering ensures all prior writes to the object
        // are visible to other threads when they see NO_OWNER.
        (*header).owner_tid.store(NO_OWNER, Ordering::Release);
    }
}

pub unsafe fn circ_destroy(header: *mut CircHeader) {
    let obj_ptr = (header as *mut u8).add(CircHeader::SIZE);
    // eprintln!("circ_destroy: {:?}", obj_ptr);

    let flags = (*header).flags.load(Ordering::Relaxed);
    if (flags & VTABLE_PTR) != 0 {
        let vtable_ptr_ptr = obj_ptr as *const *const crate::core::vtable::VTable;
        let vtable = *vtable_ptr_ptr;

        if !vtable.is_null() {
            if let Some(drop_fn) = (*vtable).drop_fn {
                drop_fn(obj_ptr as *mut u8);
            }
        }
        
        // Free inline property map for objects
        let props_slot = unsafe { obj_ptr.add(8) as *mut *mut std::collections::HashMap<String, u64> };
        crate::objects::dynamic_props::free_inline_properties(props_slot);
    }
    
    if (flags & WEAKREF_TARGET) != 0 {
        crate::weak_ref::nullify_weak_refs(header);
    }
    if (flags & FINALIZER_TARGET) != 0 {
        crate::finalization::enqueue_finalizers(header);
    }
    if (flags & (IS_CLOSURE | IS_GENERATOR)) != 0 {
        let drop_fn_ptr = *(obj_ptr.add(8) as *const *const u8);
        if !drop_fn_ptr.is_null() {
            let drop_fn: unsafe extern "C-unwind" fn(*mut u8) = std::mem::transmute(drop_fn_ptr);
            drop_fn(obj_ptr);
        }
    }

    if (flags & IS_ARRAY) != 0 {
        crate::array::free_array_data(obj_ptr);
    }
    
    // Always clean up dynamic properties (they are tracked by object pointer address)
    crate::objects::dynamic_props::remove_dynamic_properties(obj_ptr);

    crate::verify::__bs_verify_track_free(obj_ptr);

    // Free using fast thread-local cache if applicable
    let size = (*header).alloc_size;
    crate::slab::fast_free_shared(header as *mut u8, size);
}

// ── Color manipulation helpers ─────────────────────────────────────────────

impl CircHeader {
    pub fn get_color(&self) -> u16 {
        self.flags.load(Ordering::Relaxed) & COLOR_MASK
    }

    pub fn set_color(&self, color: u16) {
        let mut old = self.flags.load(Ordering::Relaxed);
        loop {
            let new_flags = (old & !COLOR_MASK) | color;
            match self.flags.compare_exchange_weak(old, new_flags, Ordering::AcqRel, Ordering::Relaxed) {
                Ok(_) => break,
                Err(e) => old = e,
            }
        }
    }
    
    pub fn is_buffered(&self) -> bool {
        self.get_color() == COLOR_PURPLE
    }
}

// ── Utility functions (moved from gc.rs) ───────────────────────────────────

/// NaN-box a `f64` number. Canonicalises NaN to avoid tag collisions.
pub fn box_number(n: f64) -> u64 {
    let mut bits = n.to_bits();
    // Canonicalise NaN to the quiet NaN that doesn't collide with our tags
    if bits & 0x7FF0_0000_0000_0000 == 0x7FF0_0000_0000_0000
        && bits & 0x000F_FFFF_FFFF_FFFF != 0
    {
        bits = 0x7FF8_0000_0000_0000;
    }
    bits
}

/// NaN-box a boolean.
pub fn box_boolean(b: bool) -> u64 {
    if b {
        0xFFF4_0000_0000_0000
    } else {
        0xFFF3_0000_0000_0000
    }
}

pub fn is_managed_ptr(val: u64) -> bool {
    let tag = val >> 48;
    tag == 0xFFF6 || tag == 0xFFF9 || tag == 0xFFFA || tag == 0xFFFB
}

#[no_mangle]
pub unsafe extern "C-unwind" fn circ_inc_tagged(val: u64) {
    let rc_tag = val >> 48;
    if rc_tag == 0xFFF6 || rc_tag == 0xFFFA || rc_tag == 0xFFF9 || rc_tag == 0xFFFB || rc_tag == 0xFFFE || rc_tag == 0x7FF6 || rc_tag == 0x7FFA || rc_tag == 0x7FF9 || rc_tag == 0x7FFB || rc_tag == 0x7FFE {
        let unbox_ptr = val & 0x0000_FFFF_FFFF_FFFF;
        let obj_ptr = unbox_ptr as *mut u8;
        if !obj_ptr.is_null() {
            let header_ptr = obj_ptr.sub(crate::circ::CircHeader::SIZE) as *mut crate::circ::CircHeader;
            crate::circ::circ_inc(header_ptr);
        }
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn circ_dec_tagged(val: u64) {
    let rc_tag = val >> 48;
    if rc_tag == 0xFFF6 || rc_tag == 0xFFFA || rc_tag == 0xFFF9 || rc_tag == 0xFFFB || rc_tag == 0xFFFE || rc_tag == 0x7FF6 || rc_tag == 0x7FFA || rc_tag == 0x7FF9 || rc_tag == 0x7FFB || rc_tag == 0x7FFE {
        let unbox_ptr = val & 0x0000_FFFF_FFFF_FFFF;
        let obj_ptr = unbox_ptr as *mut u8;
        if !obj_ptr.is_null() {
            let header_ptr = obj_ptr.sub(crate::circ::CircHeader::SIZE) as *mut crate::circ::CircHeader;
            crate::circ::circ_dec(header_ptr);
        }
    } else if rc_tag == 0x7FF6 || rc_tag == 0x7FF9 || rc_tag == 0x7FFB {
        let unbox_ptr = val & 0x0000_FFFF_FFFF_FFFF;
        let obj_ptr = unbox_ptr as *mut u8;
        if !obj_ptr.is_null() {
            crate::core::alloc::__bs_drop_owned(obj_ptr);
        }
    } else if rc_tag == 0x7FF7 {
        let unbox_ptr = val & 0x0000_FFFF_FFFF_FFFF;
        let obj_ptr = unbox_ptr as *mut u8;
        if !obj_ptr.is_null() {
            libc::free(obj_ptr as *mut libc::c_void);
        }
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn builtin_method_drop(obj_ptr: *mut u8) {
    let captured_tagged = *(obj_ptr.add(24) as *mut u64);
    circ_dec_tagged(captured_tagged);
}

#[no_mangle]
pub unsafe extern "C-unwind" fn builtin_method_trace(obj_ptr: *mut u8, visitor: *const ()) {
    let captured_tagged = *(obj_ptr.add(24) as *mut u64);
    let rc_tag = captured_tagged >> 48;
    if rc_tag == 0xFFF6 || rc_tag == 0xFFFA || rc_tag == 0xFFF9 || rc_tag == 0xFFFB || rc_tag == 0xFFFE || rc_tag == 0x7FF6 || rc_tag == 0x7FFA || rc_tag == 0x7FF9 || rc_tag == 0x7FFB || rc_tag == 0x7FFE {
        let unbox_ptr = captured_tagged & 0x0000_FFFF_FFFF_FFFF;
        let raw_ptr = unbox_ptr as *mut u8;
        if !raw_ptr.is_null() {
            // eprintln!("builtin_method_trace: tracing captured {:x}", captured_tagged);
            let header_ptr = raw_ptr.sub(crate::circ::CircHeader::SIZE) as *mut crate::circ::CircHeader;
            let visitor_fn: unsafe extern "C-unwind" fn(*mut crate::circ::CircHeader) = std::mem::transmute(visitor);
            visitor_fn(header_ptr);
        }
    }
}

pub unsafe fn create_builtin_method(obj_tagged: u64, func_ptr: *const u8) -> u64 {
    let closure_tagged = crate::core::alloc::__bs_alloc_closure(32);
    let closure_ptr = (closure_tagged & 0x0000_FFFF_FFFF_FFFF) as *mut u64;
    
    *closure_ptr = func_ptr as u64; // offset 0
    *(closure_ptr.add(1)) = builtin_method_drop as *const u8 as u64; // offset 8
    *(closure_ptr.add(2)) = builtin_method_trace as *const u8 as u64; // offset 16
    
    crate::circ::circ_inc_tagged(obj_tagged); 
    *(closure_ptr.add(3)) = obj_tagged; // offset 24
    
    closure_tagged
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_print_rc_stats() {
    let incs = ACTUAL_RC_INCS.load(std::sync::atomic::Ordering::Relaxed);
    let decs = ACTUAL_RC_DECS.load(std::sync::atomic::Ordering::Relaxed);
    let owned = OWNED_ALLOCS.load(std::sync::atomic::Ordering::Relaxed);
    let shared = SHARED_ALLOCS.load(std::sync::atomic::Ordering::Relaxed);
    let bypassed = BYPASSED_RC_OPS.load(std::sync::atomic::Ordering::Relaxed);
    let arena = ARENA_ALLOCS.load(std::sync::atomic::Ordering::Relaxed);
    let arenas_created = ARENAS_CREATED.load(std::sync::atomic::Ordering::Relaxed);
    
    let shared_frees = SHARED_FREES.load(std::sync::atomic::Ordering::Relaxed);
    let owned_drops = OWNED_DROPS.load(std::sync::atomic::Ordering::Relaxed);
    let arenas_destroyed = ARENAS_DESTROYED.load(std::sync::atomic::Ordering::Relaxed);
    
    println!("=== MEMORY & LIFETIME STATISTICS ===");
    println!("[ Allocations ]");
    println!("  Total Shared Objects (RC):   {}", shared);
    println!("  Total Owned Objects:         {}", owned);
    println!("  Total Arena Objects:         {}", arena);
    println!("");
    println!("[ Operations ]");
    println!("  RC Increments:               {}", incs);
    println!("  RC Decrements:               {}", decs);
    println!("  Bypassed RC Operations:      {}", bypassed);
    println!("");
    println!("[ Lifetimes & Cleanup ]");
    println!("  Shared Objects Freed (RC=0): {}", shared_frees);
    println!("  Owned Objects Dropped:       {}", owned_drops);
    println!("");
    println!("[ Arenas ]");
    println!("  Arenas Created:              {}", arenas_created);
    println!("  Arenas Destroyed:            {}", arenas_destroyed);
    println!("====================================");
}

pub static ACTUAL_RC_INCS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub static ACTUAL_RC_DECS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub static SHARED_ALLOCS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub static OWNED_ALLOCS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub static BYPASSED_RC_OPS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub static ARENA_ALLOCS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub static ARENAS_CREATED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub static SHARED_FREES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub static OWNED_DROPS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub static ARENAS_DESTROYED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_cleanup_tagged(tagged: u64) {
    let tag = tagged & crate::dynamic_call::helpers::TAG_MASK;
    if tag == crate::dynamic_call::helpers::TAG_OWNED ||
       tag == crate::dynamic_call::helpers::TAG_OWNED_CLOSURE ||
       tag == crate::dynamic_call::helpers::TAG_OWNED_ARRAY ||
       tag == crate::dynamic_call::helpers::TAG_OWNED_STRING {
        crate::core::alloc::__bs_drop_owned((tagged & crate::dynamic_call::helpers::PAYLOAD_MASK) as *mut u8);
    } else {
        circ_dec_tagged(tagged);
    }
}
