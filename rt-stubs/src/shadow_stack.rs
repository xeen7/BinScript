//! Shadow stack for GC root tracking.
//!
//! Every compiled function pushes a `ShadowFrame` onto a thread-local stack in its
//! prologue and pops it in its epilogue. Each frame points to the function's `alloca`'d
//! registers (all NaN-boxed i64 values), allowing the GC to find every live root.

use std::cell::Cell;

/// A single frame on the shadow stack.
///
/// Laid out as a C struct so codegen can construct it on the LLVM stack.
#[repr(C)]
pub struct ShadowFrame {
    /// Pointer to the previous frame (linked list).
    pub prev: *mut ShadowFrame,
    /// Number of root slots in this frame.
    pub num_roots: u32,
    /// Padding for alignment.
    pub _pad: u32,
    /// Pointer to an array of `num_roots` i64 values (the alloca'd registers).
    pub roots: *mut u64,
}

thread_local! {
    static SHADOW_STACK_TOP: Cell<*mut ShadowFrame> = const { Cell::new(std::ptr::null_mut()) };
}

/// Push a frame onto the shadow stack. Called in every function prologue.
#[no_mangle]
pub unsafe extern "C" fn __bs_shadow_push(frame: *mut ShadowFrame) {
    SHADOW_STACK_TOP.with(|top| {
        (*frame).prev = top.get();
        top.set(frame);
    });
}

/// Pop the top frame from the shadow stack. Called before every return.
#[no_mangle]
pub unsafe extern "C" fn __bs_shadow_pop() {
    SHADOW_STACK_TOP.with(|top| {
        let current = top.get();
        if !current.is_null() {
            top.set((*current).prev);
        }
    });
}

/// Get the current shadow stack top pointer (for saving before try blocks).
pub fn get_shadow_stack_top() -> *mut ShadowFrame {
    SHADOW_STACK_TOP.with(|top| top.get())
}

/// Restore the shadow stack top pointer (for unwinding on exception throw).
#[no_mangle]
pub unsafe extern "C" fn __bs_shadow_set(top_ptr: *mut ShadowFrame) {
    SHADOW_STACK_TOP.with(|top| top.set(top_ptr));
}

/// Walk all shadow stack frames and call `callback` on each root value.
/// Used by the GC mark phase to find all stack roots.
pub unsafe fn scan_roots(mut callback: impl FnMut(u64)) {
    SHADOW_STACK_TOP.with(|top| {
        let mut frame = top.get();
        while !frame.is_null() {
            let num = (*frame).num_roots as usize;
            let roots_ptr = (*frame).roots;
            for i in 0..num {
                let val = *roots_ptr.add(i);
                callback(val);
            }
            frame = (*frame).prev;
        }
    });
}
