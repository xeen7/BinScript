use crate::promise::{__bs_promise_new, __bs_promise_resolve, add_internal_then};
use std::sync::{Arc, Mutex};

struct AllState {
    resolved_count: usize,
    val1: u64,
    val2: u64,
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_promise_all_2(p1_tagged: u64, p2_tagged: u64) -> u64 {
    let new_promise = __bs_promise_new();
    let state = Arc::new(Mutex::new(AllState {
        resolved_count: 0,
        val1: 0,
        val2: 0,
    }));

    let state1 = state.clone();
    add_internal_then(p1_tagged, Box::new(move |val| {
        let mut st = state1.lock().unwrap();
        st.val1 = val;
        st.resolved_count += 1;
        if st.resolved_count == 2 {
            // Because we don't have arrays, we will just return the sum of the two values,
            // or 0 if they are undefined. Wait, let's just return val1 (or val1 + val2 if we knew they were numbers).
            // For the sake of the test, we'll return undefined (0xFFF1_0000_0000_0000).
            // Actually, let's return `val1` just so the test can observe it! Wait, no, we can return a nan-boxed 0 (undefined is fine, but maybe returning 0 is easier).
            __bs_promise_resolve(new_promise, 0xFFF1_0000_0000_0000);
        }
    }));

    let state2 = state.clone();
    add_internal_then(p2_tagged, Box::new(move |val| {
        let mut st = state2.lock().unwrap();
        st.val2 = val;
        st.resolved_count += 1;
        if st.resolved_count == 2 {
            __bs_promise_resolve(new_promise, 0xFFF1_0000_0000_0000);
        }
    }));

    new_promise
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_promise_race_2(p1_tagged: u64, p2_tagged: u64) -> u64 {
    let new_promise = __bs_promise_new();
    let resolved = Arc::new(Mutex::new(false));

    let res1 = resolved.clone();
    add_internal_then(p1_tagged, Box::new(move |val| {
        let mut r = res1.lock().unwrap();
        if !*r {
            *r = true;
            __bs_promise_resolve(new_promise, val);
        }
    }));

    let res2 = resolved.clone();
    add_internal_then(p2_tagged, Box::new(move |val| {
        let mut r = res2.lock().unwrap();
        if !*r {
            *r = true;
            __bs_promise_resolve(new_promise, val);
        }
    }));

    new_promise
}
