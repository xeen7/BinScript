use std::cell::RefCell;

pub struct ScopeGuard {
    pub scope_id: u32,
    pub obj_val: u64,
    pub release_fn: unsafe extern "C-unwind" fn(u64, u64) -> u64,
}

thread_local! {
    static GUARD_STACK: RefCell<Vec<ScopeGuard>> = RefCell::new(Vec::new());
}

#[no_mangle]
pub extern "C-unwind" fn __bs_scope_guard_get_len() -> u32 {
    GUARD_STACK.with(|stack| stack.borrow().len() as u32)
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_scope_guard_push(
    scope_id: u32,
    obj_val: u64,
    release_fn: unsafe extern "C-unwind" fn(u64, u64) -> u64,
) {
    GUARD_STACK.with(|stack| {
        stack.borrow_mut().push(ScopeGuard {
            scope_id,
            obj_val,
            release_fn,
        });
    });
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_scope_guard_cancel(frame_base: u32, scope_id: u32, obj_val: u64) {
    GUARD_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        for i in (frame_base as usize..stack.len()).rev() {
            if stack[i].scope_id == scope_id && stack[i].obj_val == obj_val {
                stack.remove(i);
                break;
            }
        }
    });
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_scope_guard_flush_to(frame_base: u32, target_scope_id: u32) {
    GUARD_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        while stack.len() > frame_base as usize {
            if let Some(guard) = stack.last() {
                if guard.scope_id >= target_scope_id {
                    let guard = stack.pop().unwrap();
                    let _ = (guard.release_fn)(0xFFF1000000000000, guard.obj_val);
                } else {
                    break;
                }
            } else { break; }
        }
    });
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_scope_guard_flush_all() {
    GUARD_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        while let Some(guard) = stack.pop() {
            let _ = (guard.release_fn)(0xFFF1000000000000, guard.obj_val);
        }
    });
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_scope_guard_get_depth() -> usize {
    GUARD_STACK.with(|stack| stack.borrow().len())
}

/// Flush scope guards down to a specific depth. Called by landing pads during
/// exception unwinding to clean up resources in callee frames.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_scope_guard_flush_down_to(depth: u32) {
    GUARD_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        while stack.len() > depth as usize {
            if let Some(guard) = stack.pop() {
                let _ = (guard.release_fn)(0xFFF1000000000000, guard.obj_val);
            }
        }
    });
}
