use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::sync::Mutex;

#[repr(C)]
pub struct GeneratorState {
    pub poll_fn: extern "C-unwind" fn(*mut GeneratorState, u64) -> u64,
    pub drop_fn: extern "C-unwind" fn(*mut u8),
    pub trace_fn: extern "C-unwind" fn(*mut u8, *const ()),
    pub state_idx: i64,
}

#[derive(Clone, Copy)]
pub struct ArrayIteratorState {
    pub index: usize,
    pub done: bool,
}

pub static ARRAY_ITERATORS: Lazy<Mutex<HashMap<usize, ArrayIteratorState>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_generator_next(gen_tagged: u64, sent_value: u64) -> u64 {
    let tag = gen_tagged & 0xFFFF_0000_0000_0000;
    if crate::dynamic_call::helpers::is_array_tag(gen_tagged) {
        let arr_ptr = (gen_tagged & 0x0000_FFFF_FFFF_FFFF) as *mut crate::array::BsArray;
        if arr_ptr.is_null() {
            return 0xFFF1_0000_0000_0000; // undefined
        }
        let mut map = ARRAY_ITERATORS.lock().unwrap();
        let state = map.entry(arr_ptr as usize).or_insert(ArrayIteratorState { index: 0, done: false });
        let len = (*arr_ptr).length as usize;
        if state.index >= len {
            state.done = true;
            return 0xFFF1_0000_0000_0000; // undefined
        }
        let val = *(*arr_ptr).data.add(state.index);
        state.index += 1;
        return val;
    }
    if tag != 0xFFFA_0000_0000_0000 {
        panic!("__bs_generator_next called on non-generator (tag: {:X})", tag);
    }
    let ptr = (gen_tagged & 0x0000_FFFF_FFFF_FFFF) as *mut GeneratorState;
    if (*ptr).state_idx == -1 {
        // Return undefined if generator is already exhausted
        return 0;
    }
    let poll = (*ptr).poll_fn;
    poll(ptr, sent_value)
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_generator_is_done(gen_tagged: u64) -> u64 {
    let tag = gen_tagged & 0xFFFF_0000_0000_0000;
    if crate::dynamic_call::helpers::is_array_tag(gen_tagged) {
        let arr_ptr = (gen_tagged & 0x0000_FFFF_FFFF_FFFF) as *mut crate::array::BsArray;
        if arr_ptr.is_null() {
            return 0xFFF4_0000_0000_0000; // true (done)
        }
        let mut map = ARRAY_ITERATORS.lock().unwrap();
        let state = map.entry(arr_ptr as usize).or_insert(ArrayIteratorState { index: 0, done: false });
        if state.done {
            map.remove(&(arr_ptr as usize));
            return 0xFFF4_0000_0000_0000; // true
        } else {
            return 0xFFF3_0000_0000_0000; // false
        }
    }
    if tag != 0xFFFA_0000_0000_0000 {
        return 0xFFF4_0000_0000_0000; // Treat non-generators as done (true)
    }
    let ptr = (gen_tagged & 0x0000_FFFF_FFFF_FFFF) as *mut GeneratorState;
    if (*ptr).state_idx == -1 {
        // Nano-box bool true
        0xFFF4_0000_0000_0000
    } else {
        // Nano-box bool false
        0xFFF3_0000_0000_0000
    }
}
