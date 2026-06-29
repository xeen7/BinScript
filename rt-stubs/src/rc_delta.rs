use crate::circ::{CircHeader, circ_inc, circ_dec};
use std::collections::HashMap;
use std::cell::RefCell;

thread_local! {
    static DELTA_BUFFER: RefCell<HashMap<*mut CircHeader, i32>> = RefCell::new(HashMap::new());
}

/// Increment the reference count in the thread-local deferred buffer.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_rc_inc_deferred(header: *mut CircHeader) {
    if header.is_null() { return; }
    let obj_ptr = (header as *mut u8).add(CircHeader::SIZE);
    // println!("__bs_rc_inc_deferred: {:?}", obj_ptr);
    DELTA_BUFFER.with(|buf| {
        *buf.borrow_mut().entry(header).or_insert(0) += 1;
    });
}

/// Decrement the reference count in the thread-local deferred buffer.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_rc_dec_deferred(header: *mut CircHeader) {
    if header.is_null() { return; }
    let obj_ptr = (header as *mut u8).add(CircHeader::SIZE);
    // println!("__bs_rc_dec_deferred: {:?}", obj_ptr);
    DELTA_BUFFER.with(|buf| {
        *buf.borrow_mut().entry(header).or_insert(0) -= 1;
    });
}

/// Flush all deferred reference count operations to the actual objects.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_rc_flush() {
    DELTA_BUFFER.with(|buf| {
        let mut b = buf.borrow_mut();
        for (&header, &delta) in b.iter() {
            if delta > 0 {
                for _ in 0..delta {
                    circ_inc(header);
                }
            } else if delta < 0 {
                for _ in 0..(-delta) {
                    circ_dec(header);
                }
            }
        }
        b.clear();
    });
}
