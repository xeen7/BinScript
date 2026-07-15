pub mod combinators;
pub mod microtask;
pub mod reactor;
use crate::promise::microtask::enqueue_microtask;
use std::sync::Mutex;

pub const TAG_PROMISE: u64 = 0xFFFD_0000_0000_0000;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PromiseState {
    Pending,
    Fulfilled(u64),
    Rejected(u64),
}

type PromiseCallback = Box<dyn FnOnce(u64) + Send + 'static>;

pub struct Promise {
    pub state: PromiseState,
    pub then_callbacks: Vec<PromiseCallback>,
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_promise_new() -> u64 {
    let p = Box::new(Mutex::new(Promise {
        state: PromiseState::Pending,
        then_callbacks: Vec::new(),
    }));
    let ptr = Box::into_raw(p);
    let promise_tagged = (ptr as u64) | TAG_PROMISE;
    promise_tagged
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_promise_static_resolve(value: u64) -> u64 {
    let p = __bs_promise_new();
    __bs_promise_resolve(p, value);
    p
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_promise_resolve(promise_tagged: u64, value: u64) {
    let tag = promise_tagged & 0xFFFF_0000_0000_0000;
    if tag != TAG_PROMISE {
        return;
    }
    let ptr = (promise_tagged & 0x0000_FFFF_FFFF_FFFF) as *mut Mutex<Promise>;
    let mut p = (*ptr).lock().unwrap();
    if p.state != PromiseState::Pending {
        return;
    }
    p.state = PromiseState::Fulfilled(value);
    
    let callbacks = std::mem::take(&mut p.then_callbacks);
    for cb in callbacks {
        enqueue_microtask(move || {
            cb(value);
        });
    }
    
    // Wake up any threads blocked waiting for tasks (e.g., the main thread waiting on this root promise)
    crate::promise::microtask::wake_all_microtasks();
}

pub unsafe fn add_internal_then(promise_tagged: u64, cb: PromiseCallback) {
    let ptr = (promise_tagged & 0x0000_FFFF_FFFF_FFFF) as *mut Mutex<Promise>;
    let mut p = (*ptr).lock().unwrap();
    match p.state {
        PromiseState::Pending => {
            p.then_callbacks.push(cb);
        }
        PromiseState::Fulfilled(val) => {
            enqueue_microtask(move || {
                cb(val);
            });
        }
        PromiseState::Rejected(_) => {}
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_promise_then(promise_tagged: u64, callback_closure: u64) -> u64 {
    let closure_ptr = (callback_closure & 0x0000_FFFF_FFFF_FFFF) as *const u64;
    let cb_fn: extern "C-unwind" fn(u64, u64, u64) -> u64 = std::mem::transmute(*closure_ptr);
    
    let new_promise = __bs_promise_new();
    
    add_internal_then(promise_tagged, Box::new(move |val| {
        let res = cb_fn(callback_closure, 0xFFF1_0000_0000_0000, val);
        __bs_promise_resolve(new_promise, res);
    }));
    
    new_promise
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_async_drive(gen_tagged: u64) -> u64 {
    let return_promise = __bs_promise_new();
    __bs_async_step(gen_tagged, 0, return_promise);
    return_promise
}

unsafe fn __bs_async_step(gen_tagged: u64, sent_val: u64, return_promise: u64) {
    let yielded_val = crate::__bs_generator_next(gen_tagged, sent_val);
    
    let ptr = (gen_tagged & 0x0000_FFFF_FFFF_FFFF) as *mut crate::GeneratorState;
    if (*ptr).state_idx == -1 {
        __bs_promise_resolve(return_promise, yielded_val);
        crate::circ::circ_dec_tagged(gen_tagged);
        return;
    }
    
    let is_promise = (yielded_val & 0xFFFF_0000_0000_0000) == TAG_PROMISE;
    if is_promise {
        add_internal_then(yielded_val, Box::new(move |val| {
            unsafe { __bs_async_step(gen_tagged, val, return_promise); }
        }));
    } else {
        // Yielded a non-promise, just continue synchronously
        enqueue_microtask(move || {
            unsafe { __bs_async_step(gen_tagged, yielded_val, return_promise); }
        });
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_promise_state(promise_tagged: u64) -> u32 {
    let ptr = (promise_tagged & 0x0000_FFFF_FFFF_FFFF) as *mut Mutex<Promise>;
    let p = (*ptr).lock().unwrap();
    match p.state {
        PromiseState::Pending => 0,
        PromiseState::Fulfilled(_) => 1,
        PromiseState::Rejected(_) => 2,
    }
}
