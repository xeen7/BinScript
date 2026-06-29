use crate::VTable;
use crate::dynamic_call::helpers::{TAG_MASK, PAYLOAD_MASK};

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_call_apply(callee: u64, _this_val: u64, args_array: u64) -> u64 {
    let tag = callee & TAG_MASK;
    if tag != 0xFFF9_0000_0000_0000 {
        panic!("__bs_call_apply: callee is not a closure (tag: {:X})", tag);
    }
    
    let closure_ptr = (callee & PAYLOAD_MASK) as *const u64;
    let fn_ptr = *closure_ptr;
    if fn_ptr == 0 {
        panic!("__bs_call_apply: closure has null function pointer");
    }
    
    let len_boxed = crate::array::__bs_array_length(args_array);
    let len = f64::from_bits(len_boxed) as usize;
    let mut args = Vec::new();
    for i in 0..len {
        let idx = crate::circ::box_number(i as f64);
        args.push(crate::array::__bs_array_get(args_array, idx));
    }

    match len {
        0 => {
            let cb: unsafe extern "C-unwind" fn(u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(callee)
        }
        1 => {
            let cb: unsafe extern "C-unwind" fn(u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(callee, args[0])
        }
        2 => {
            let cb: unsafe extern "C-unwind" fn(u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(callee, args[0], args[1])
        }
        3 => {
            let cb: unsafe extern "C-unwind" fn(u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(callee, args[0], args[1], args[2])
        }
        4 => {
            let cb: unsafe extern "C-unwind" fn(u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(callee, args[0], args[1], args[2], args[3])
        }
        5 => {
            let cb: unsafe extern "C-unwind" fn(u64, u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(callee, args[0], args[1], args[2], args[3], args[4])
        }
        6 => {
            let cb: unsafe extern "C-unwind" fn(u64, u64, u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(callee, args[0], args[1], args[2], args[3], args[4], args[5])
        }
        7 => {
            let cb: unsafe extern "C-unwind" fn(u64, u64, u64, u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(callee, args[0], args[1], args[2], args[3], args[4], args[5], args[6])
        }
        8 => {
            let cb: unsafe extern "C-unwind" fn(u64, u64, u64, u64, u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(callee, args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7])
        }
        _ => {
            panic!("__bs_call_apply: dynamic call with {} arguments is unsupported (max 8)", len);
        }
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_vcall_apply(obj: u64, method_idx_val: u64, args_array: u64) -> u64 {
    let method_idx = f64::from_bits(method_idx_val) as i32;
    let payload = obj & PAYLOAD_MASK;
    if payload == 0 {
        panic!("__bs_vcall_apply: obj is null");
    }
    
    let obj_ptr = payload as *const *const VTable;
    let vtable_ptr = *obj_ptr;
    if vtable_ptr.is_null() {
        panic!("__bs_vcall_apply: vtable is null");
    }
    
    // Look up method pointer in the vtable chain
    let mut current_vtable = vtable_ptr;
    let mut fn_ptr: *const u8 = std::ptr::null();
    while !current_vtable.is_null() {
        let slot = *(current_vtable as *const *const u8).add(5 + method_idx as usize);
        if !slot.is_null() {
            fn_ptr = slot;
            break;
        }
        current_vtable = (*current_vtable).parent;
    }
    if fn_ptr.is_null() {
        panic!("__bs_vcall_apply: method not found in vtable (idx: {})", method_idx);
    }
        let len_boxed = crate::array::__bs_array_length(args_array);
    let len = f64::from_bits(len_boxed) as usize;
    let mut args = Vec::new();
    for i in 0..len {
        let idx = crate::circ::box_number(i as f64);
        args.push(crate::array::__bs_array_get(args_array, idx));
    }

    // Note: class methods expect (this, arg1, arg2...)
    // Since args_array already has `obj` prepended as its first element,
    // args[0] is the receiver `this`, and args[1..] are the subsequent arguments.
    match len {
        1 => {
            let cb: unsafe extern "C-unwind" fn(u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(args[0])
        }
        2 => {
            let cb: unsafe extern "C-unwind" fn(u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(args[0], args[1])
        }
        3 => {
            let cb: unsafe extern "C-unwind" fn(u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(args[0], args[1], args[2])
        }
        4 => {
            let cb: unsafe extern "C-unwind" fn(u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(args[0], args[1], args[2], args[3])
        }
        5 => {
            let cb: unsafe extern "C-unwind" fn(u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(args[0], args[1], args[2], args[3], args[4])
        }
        6 => {
            let cb: unsafe extern "C-unwind" fn(u64, u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(args[0], args[1], args[2], args[3], args[4], args[5])
        }
        7 => {
            let cb: unsafe extern "C-unwind" fn(u64, u64, u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(args[0], args[1], args[2], args[3], args[4], args[5], args[6])
        }
        8 => {
            let cb: unsafe extern "C-unwind" fn(u64, u64, u64, u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7])
        }
        9 => {
            let cb: unsafe extern "C-unwind" fn(u64, u64, u64, u64, u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
            cb(args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7], args[8])
        }
        _ => {
            panic!("__bs_vcall_apply: dynamic call with {} arguments is unsupported (max 8 actual args)", len - 1);
        }
    }
}
