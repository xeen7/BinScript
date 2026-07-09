use std::cell::RefCell;
use crate::circ::{CircHeader, COLOR_PURPLE};
use crate::cycle_collector;

const CYCLE_BUFFER_HWM: usize = 512;

thread_local! {
    static LOCAL_BUFFER: RefCell<Vec<*mut CircHeader>> = RefCell::new(Vec::with_capacity(CYCLE_BUFFER_HWM));
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_cycle_buffer_push(header_ptr: *mut CircHeader) {
    let header = &*header_ptr;
    
    // Only buffer if it's not already buffered
    if !header.is_buffered() {
        header.set_color(COLOR_PURPLE);
        
        LOCAL_BUFFER.with(|buf| {
        let b_len = buf.borrow().len();
        eprintln!("flush_local: buffer len={}", b_len);
            let mut b = buf.borrow_mut();
            b.push(header_ptr);
            
            if b.len() >= CYCLE_BUFFER_HWM {
                let mut flush_vec = std::mem::take(&mut *b);
                eprintln!("flush_local: pushing {} items", flush_vec.len());
            cycle_collector::push_to_global_queue(&mut flush_vec);
            }
        });
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_cycle_buffer_flush_local() {
    LOCAL_BUFFER.with(|buf| {
        let b_len = buf.borrow().len();
        eprintln!("flush_local: buffer len={}", b_len);
        let mut b = buf.borrow_mut();
        if !b.is_empty() {
            let mut flush_vec = std::mem::take(&mut *b);
            eprintln!("flush_local: pushing {} items", flush_vec.len());
            cycle_collector::push_to_global_queue(&mut flush_vec);
        }
    });
}
