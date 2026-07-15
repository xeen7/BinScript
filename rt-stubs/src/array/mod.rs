//! Array runtime support for BinScript.
//!
//! Arrays are CIRC-allocated resizable buffers of NaN-boxed values.
//! Tagged with TAG_ARRAY = 0xFFFB_0000_0000_0000.

use std::sync::atomic::Ordering;


const TAG_ARRAY: u64 = 0xFFFB_0000_0000_0000;
const TAG_MASK: u64 = 0xFFFF_0000_0000_0000;
const PAYLOAD_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;

/// In-memory layout of a BsArray.
/// This struct lives behind a CircHeader.
#[repr(C)]
pub struct BsArray {
    /// Number of elements currently stored.
    pub length: u32,
    /// Allocated capacity (in elements, not bytes).
    pub capacity: u32,
    /// Pointer to the element buffer (each element is a u64 NaN-boxed value).
    pub data: *mut u64,
}

/// Extract the raw BsArray pointer from a tagged value.
/// Returns null if the tag doesn't match TAG_ARRAY.
pub(crate) unsafe fn untag_array(tagged: u64) -> *mut BsArray {
    let tag = tagged & TAG_MASK;
    if tag != TAG_ARRAY && tag != crate::dynamic_call::helpers::TAG_OWNED_ARRAY && tag != crate::dynamic_call::helpers::TAG_ARENA_ARRAY {
        return std::ptr::null_mut();
    }
    let payload = tagged & PAYLOAD_MASK;
    payload as *mut BsArray
}

/// Grow the element buffer to at least `new_cap` elements.
pub(crate) unsafe fn grow_array(arr: *mut BsArray, new_cap: u32) {
    let old_cap = (*arr).capacity;
    if new_cap <= old_cap {
        return;
    }
    let cap = std::cmp::max(new_cap, old_cap * 2).max(8);
    let new_size = (cap as usize) * std::mem::size_of::<u64>();
    let new_data = if (*arr).data.is_null() {
        libc::malloc(new_size) as *mut u64
    } else {
        libc::realloc((*arr).data as *mut libc::c_void, new_size) as *mut u64
    };
    if new_data.is_null() {
        panic!("Array: out of memory");
    }
    // Zero new slots
    for i in old_cap..cap {
        *new_data.add(i as usize) = 0; // undefined
    }
    (*arr).data = new_data;
    (*arr).capacity = cap;
}

// ===========================================================================
// Public API — called from compiled code via `extern "C-unwind"`
// ===========================================================================

/// Create an empty array.  Returns a NaN-boxed TAG_ARRAY pointer.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_new() -> u64 {
    let size = std::mem::size_of::<BsArray>();
    let total_size = crate::circ::CircHeader::SIZE + size;

    let (raw, alloc_size) = crate::slab::fast_alloc_shared(total_size);

    let header = raw as *mut crate::circ::CircHeader;
    (*header).local_rc = 1;
    (*header).global_rc.store(0, Ordering::Relaxed);
    (*header).owner_tid.store(crate::circ::current_thread_id(), Ordering::Relaxed);
    (*header).flags.store(crate::circ::IS_ARRAY, Ordering::Relaxed);
    (*header).alloc_size = alloc_size;
    (*header).crc = 0;

    let ptr = (raw as *mut u8).add(crate::circ::CircHeader::SIZE);
    let arr = ptr as *mut BsArray;
    (*arr).length = 0;
    (*arr).capacity = 0;
    (*arr).data = std::ptr::null_mut();
    crate::verify::__bs_verify_track_alloc(ptr);
    crate::circ::SHARED_ALLOCS.fetch_add(1, Ordering::Relaxed);
    println!("Allocated Shared Array: {:p}", ptr);
    (ptr as u64) | TAG_ARRAY
}

/// Create an array from `count` values pushed on the stack.
/// The caller passes a pointer to a contiguous block of NaN-boxed u64 values.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_from(elements: *const u64, count: u32) -> u64 {
    let tagged = __bs_array_new();
    let arr = untag_array(tagged);
    if count > 0 {
        grow_array(arr, count);
        std::ptr::copy_nonoverlapping(elements, (*arr).data, count as usize);
        (*arr).length = count;
    }
    tagged
}

/// `arr.push(val)` — appends a value, returns new length as f64.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_push(arr_tagged: u64, val: u64) -> u64 {
    let arr = untag_array(arr_tagged);
    if arr.is_null() { return 0; }
    crate::circ::circ_inc_tagged(val);
    let len = (*arr).length;
    grow_array(arr, len + 1);
    *(*arr).data.add(len as usize) = val;
    (*arr).length = len + 1;
    // Return new length as f64
    crate::circ::box_number((len + 1) as f64)
}

/// Push all elements/chars from `spread_val` into `arr_tagged`.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_push_spread(arr_tagged: u64, spread_val: u64) -> u64 {
    let arr = untag_array(arr_tagged);
    if arr.is_null() { return 0; }
    
    let tag = spread_val & TAG_MASK;
    if tag == TAG_ARRAY || tag == crate::dynamic_call::helpers::TAG_OWNED_ARRAY || tag == crate::dynamic_call::helpers::TAG_ARENA_ARRAY {
        let src_arr = untag_array(spread_val);
        if !src_arr.is_null() {
            let src_len = (*src_arr).length;
            if src_len > 0 {
                let dest_len = (*arr).length;
                grow_array(arr, dest_len + src_len);
                for i in 0..src_len as usize {
                    let val = *(*src_arr).data.add(i);
                    crate::circ::circ_inc_tagged(val);
                    *(*arr).data.add(dest_len as usize + i) = val;
                }
                (*arr).length = dest_len + src_len;
            }
        }
    } else if tag == 0xFFF7_0000_0000_0000 {
        let s = crate::get_c_string_from_tagged(spread_val);
        for c in s.chars() {
            let char_str = c.to_string();
            let char_tagged = crate::create_tagged_string(&char_str);
            __bs_array_push(arr_tagged, char_tagged);
        }
    }
    
    crate::circ::box_number((*arr).length as f64)
}

/// `arr.pop()` — removes and returns the last element.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_pop(arr_tagged: u64) -> u64 {
    let arr = untag_array(arr_tagged);
    if arr.is_null() || (*arr).length == 0 { return 0xFFF1_0000_0000_0000; /* undefined */ }
    (*arr).length -= 1;
    let val = *(*arr).data.add((*arr).length as usize);
    *(*arr).data.add((*arr).length as usize) = 0xFFF1_0000_0000_0000; // clear for GC
    val
}

/// `arr[index]` — get element at index. Returns undefined if out of bounds.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_get(arr_tagged: u64, index_tagged: u64) -> u64 {
    let arr = untag_array(arr_tagged);
    if arr.is_null() { return 0xFFF1_0000_0000_0000; }
    let idx = f64::from_bits(index_tagged) as i64;
    if idx < 0 || idx >= (*arr).length as i64 {
        return 0xFFF1_0000_0000_0000; // undefined
    }
    let val = *(*arr).data.add(idx as usize);
    crate::circ::circ_inc_tagged(val);
    val
}

/// `arr[index] = val` — set element at index.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_set(arr_tagged: u64, index_tagged: u64, val: u64) {
    let arr = untag_array(arr_tagged);
    if arr.is_null() { return; }
    let idx = f64::from_bits(index_tagged) as u32;
    if idx >= (*arr).capacity {
        grow_array(arr, idx + 1);
    }
    if idx >= (*arr).length {
        (*arr).length = idx + 1;
    }
    crate::circ::circ_inc_tagged(val);
    let old_val = *(*arr).data.add(idx as usize);
    if old_val != 0 && old_val != 0xFFF1_0000_0000_0000 {
        crate::circ::circ_dec_tagged(old_val);
    }
    *(*arr).data.add(idx as usize) = val;
}

/// `arr.length` — returns length as NaN-boxed f64.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_length(arr_tagged: u64) -> u64 {
    let arr = untag_array(arr_tagged);
    if arr.is_null() { return crate::circ::box_number(0.0); }
    crate::circ::box_number((*arr).length as f64)
}

/// `arr.slice(start?, end?)` — returns a shallow copy of a portion.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_slice(arr_tagged: u64, start_tagged: u64, end_tagged: u64) -> u64 {
    let arr = untag_array(arr_tagged);
    if arr.is_null() { return __bs_array_new(); }
    let len = (*arr).length as i64;

    let start = if start_tagged == 0 || start_tagged == 0xFFF1_0000_0000_0000 { 0 } else {
        let s = f64::from_bits(start_tagged) as i64;
        if s < 0 { std::cmp::max(len + s, 0) } else { std::cmp::min(s, len) }
    };
    let end = if end_tagged == 0 || end_tagged == 0xFFF1_0000_0000_0000 { len } else {
        let e = f64::from_bits(end_tagged) as i64;
        if e < 0 { std::cmp::max(len + e, 0) } else { std::cmp::min(e, len) }
    };

    let new_tagged = __bs_array_new();
    if start < end {
        let new_arr = untag_array(new_tagged);
        let count = (end - start) as u32;
        grow_array(new_arr, count);
        for i in 0..count as usize {
            let val = *(*arr).data.add(start as usize + i);
            crate::circ::circ_inc_tagged(val);
            *(*new_arr).data.add(i) = val;
        }
        (*new_arr).length = count;
    }
    new_tagged
}

/// `arr.indexOf(val)` — returns index as f64 or -1.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_indexOf(arr_tagged: u64, val: u64) -> u64 {
    let arr = untag_array(arr_tagged);
    if arr.is_null() { return crate::circ::box_number(-1.0); }
    for i in 0..(*arr).length as usize {
        if *(*arr).data.add(i) == val {
            return crate::circ::box_number(i as f64);
        }
    }
    crate::circ::box_number(-1.0)
}

/// `arr.includes(val)` — returns boolean.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_includes(arr_tagged: u64, val: u64) -> u64 {
    let arr = untag_array(arr_tagged);
    if arr.is_null() { return 0xFFF3_0000_0000_0000; }
    for i in 0..(*arr).length as usize {
        if *(*arr).data.add(i) == val {
            return 0xFFF4_0000_0000_0000; // true
        }
    }
    0xFFF3_0000_0000_0000 // false
}

/// `arr.join(sep)` — joins elements with separator string, returns tagged string.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_join(arr_tagged: u64, sep_tagged: u64) -> u64 {
    let arr = untag_array(arr_tagged);
    if arr.is_null() { return crate::create_tagged_string(""); }
    let sep = if sep_tagged == 0 {
        ","
    } else {
        crate::get_c_string_from_tagged(sep_tagged)
    };
    let mut parts = Vec::new();
    for i in 0..(*arr).length as usize {
        let val = *(*arr).data.add(i);
        parts.push(value_to_string(val));
    }
    crate::create_tagged_string(&parts.join(sep))
}

/// `arr.reverse()` — reverses in place, returns the array.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_reverse(arr_tagged: u64) -> u64 {
    let arr = untag_array(arr_tagged);
    if arr.is_null() || (*arr).length <= 1 { return arr_tagged; }
    let len = (*arr).length as usize;
    let data = (*arr).data;
    for i in 0..len / 2 {
        let j = len - 1 - i;
        let tmp = *data.add(i);
        *data.add(i) = *data.add(j);
        *data.add(j) = tmp;
    }
    arr_tagged
}

/// `arr.concat(other)` — returns a new array with elements from both.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_concat(a_tagged: u64, b_tagged: u64) -> u64 {
    let a = untag_array(a_tagged);
    let b = untag_array(b_tagged);
    let new_tagged = __bs_array_new();
    let new_arr = untag_array(new_tagged);

    let a_len = if a.is_null() { 0 } else { (*a).length };
    let b_len = if b.is_null() { 0 } else { (*b).length };
    let total = a_len + b_len;

    if total > 0 {
        grow_array(new_arr, total);
        if a_len > 0 {
            for i in 0..a_len as usize {
                let val = *(*a).data.add(i);
                crate::circ::circ_inc_tagged(val);
                *(*new_arr).data.add(i) = val;
            }
        }
        if b_len > 0 {
            for i in 0..b_len as usize {
                let val = *(*b).data.add(i);
                crate::circ::circ_inc_tagged(val);
                *(*new_arr).data.add(a_len as usize + i) = val;
            }
        }
        (*new_arr).length = total;
    }
    new_tagged
}

/// `arr.fill(val, start?, end?)` — fills with a value, returns the array.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_fill(arr_tagged: u64, val: u64, start_tagged: u64, end_tagged: u64) -> u64 {
    let arr = untag_array(arr_tagged);
    if arr.is_null() { return arr_tagged; }
    let len = (*arr).length as i64;
    let start = if start_tagged == 0 { 0i64 } else {
        let s = f64::from_bits(start_tagged) as i64;
        if s < 0 { std::cmp::max(len + s, 0) } else { s }
    };
    let end = if end_tagged == 0 { len } else {
        let e = f64::from_bits(end_tagged) as i64;
        if e < 0 { std::cmp::max(len + e, 0) } else { std::cmp::min(e, len) }
    };
    for i in start..end {
        crate::circ::circ_inc_tagged(val);
        let old_val = *(*arr).data.add(i as usize);
        if old_val != 0 && old_val != 0xFFF1_0000_0000_0000 {
            crate::circ::circ_dec_tagged(old_val);
        }
        *(*arr).data.add(i as usize) = val;
    }
    arr_tagged
}

/// `Array.isArray(val)` — returns boolean.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_isArray(val: u64) -> u64 {
    let tag = val & TAG_MASK;
    if tag == TAG_ARRAY || tag == crate::dynamic_call::helpers::TAG_OWNED_ARRAY || tag == crate::dynamic_call::helpers::TAG_ARENA_ARRAY {
        0xFFF4_0000_0000_0000 // true
    } else {
        0xFFF3_0000_0000_0000 // false
    }
}

// ===========================================================================
// Higher-order array methods (map, filter, reduce, forEach, find, etc.)
//
// These accept a closure (TAG_CLOSURE = 0xFFF9). They call the closure
// by transmuting the function pointer stored as the first slot of the closure.
// ===========================================================================

/// `arr.forEach(fn)` — calls fn(elem, index, arr) for each element.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_forEach(arr_tagged: u64, cb_tagged: u64) {
    let arr = untag_array(arr_tagged);
    if arr.is_null() { return; }
    let closure_ptr = (cb_tagged & PAYLOAD_MASK) as *const u64;
    let cb_fn: unsafe extern "C-unwind" fn(u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(*closure_ptr);
    for i in 0..(*arr).length as usize {
        let val = *(*arr).data.add(i);
        let idx = crate::circ::box_number(i as f64);
        cb_fn(cb_tagged, 0xFFF1_0000_0000_0000, val, idx, arr_tagged);
    }
}

/// `arr.map(fn)` — returns a new array with fn(elem, index, arr) for each element.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_map(arr_tagged: u64, cb_tagged: u64) -> u64 {
    let arr = untag_array(arr_tagged);
    if arr.is_null() { return __bs_array_new(); }
    let new_tagged = __bs_array_new();
    let new_arr = untag_array(new_tagged);
    let len = (*arr).length;
    grow_array(new_arr, len);
    let closure_ptr = (cb_tagged & PAYLOAD_MASK) as *const u64;
    let cb_fn: unsafe extern "C-unwind" fn(u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(*closure_ptr);
    for i in 0..len as usize {
        let val = *(*arr).data.add(i);
        let idx = crate::circ::box_number(i as f64);
        let result = cb_fn(cb_tagged, 0xFFF1_0000_0000_0000, val, idx, arr_tagged);
        *(*new_arr).data.add(i) = result;
    }
    (*new_arr).length = len;
    new_tagged
}

/// `arr.filter(fn)` — returns a new array with elements where fn returns truthy.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_filter(arr_tagged: u64, cb_tagged: u64) -> u64 {
    let arr = untag_array(arr_tagged);
    if arr.is_null() { return __bs_array_new(); }
    let new_tagged = __bs_array_new();
    let closure_ptr = (cb_tagged & PAYLOAD_MASK) as *const u64;
    let cb_fn: unsafe extern "C-unwind" fn(u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(*closure_ptr);
    for i in 0..(*arr).length as usize {
        let val = *(*arr).data.add(i);
        let idx = crate::circ::box_number(i as f64);
        let result = cb_fn(cb_tagged, 0xFFF1_0000_0000_0000, val, idx, arr_tagged);
        if is_truthy(result) {
            __bs_array_push(new_tagged, val);
        }
    }
    new_tagged
}

/// `arr.find(fn)` — returns the first element where fn returns truthy.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_find(arr_tagged: u64, cb_tagged: u64) -> u64 {
    let arr = untag_array(arr_tagged);
    if arr.is_null() { return 0; }
    let closure_ptr = (cb_tagged & PAYLOAD_MASK) as *const u64;
    let cb_fn: unsafe extern "C-unwind" fn(u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(*closure_ptr);
    for i in 0..(*arr).length as usize {
        let val = *(*arr).data.add(i);
        let idx = crate::circ::box_number(i as f64);
        let result = cb_fn(cb_tagged, 0xFFF1_0000_0000_0000, val, idx, arr_tagged);
        if is_truthy(result) {
            return val;
        }
    }
    0 // undefined
}

/// `arr.findIndex(fn)` — returns the index of the first truthy element, or -1.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_findIndex(arr_tagged: u64, cb_tagged: u64) -> u64 {
    let arr = untag_array(arr_tagged);
    if arr.is_null() { return crate::circ::box_number(-1.0); }
    let closure_ptr = (cb_tagged & PAYLOAD_MASK) as *const u64;
    let cb_fn: unsafe extern "C-unwind" fn(u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(*closure_ptr);
    for i in 0..(*arr).length as usize {
        let val = *(*arr).data.add(i);
        let idx = crate::circ::box_number(i as f64);
        let result = cb_fn(cb_tagged, 0xFFF1_0000_0000_0000, val, idx, arr_tagged);
        if is_truthy(result) {
            return crate::circ::box_number(i as f64);
        }
    }
    crate::circ::box_number(-1.0)
}

/// `arr.every(fn)` — returns true if fn returns truthy for all elements.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_every(arr_tagged: u64, cb_tagged: u64) -> u64 {
    let arr = untag_array(arr_tagged);
    if arr.is_null() { return 0xFFF4_0000_0000_0000; } // true for empty
    let closure_ptr = (cb_tagged & PAYLOAD_MASK) as *const u64;
    let cb_fn: unsafe extern "C-unwind" fn(u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(*closure_ptr);
    for i in 0..(*arr).length as usize {
        let val = *(*arr).data.add(i);
        let idx = crate::circ::box_number(i as f64);
        let result = cb_fn(cb_tagged, 0xFFF1_0000_0000_0000, val, idx, arr_tagged);
        if !is_truthy(result) {
            return 0xFFF3_0000_0000_0000; // false
        }
    }
    0xFFF4_0000_0000_0000 // true
}

/// `arr.some(fn)` — returns true if fn returns truthy for any element.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_some(arr_tagged: u64, cb_tagged: u64) -> u64 {
    let arr = untag_array(arr_tagged);
    if arr.is_null() { return 0xFFF3_0000_0000_0000; } // false for empty
    let closure_ptr = (cb_tagged & PAYLOAD_MASK) as *const u64;
    let cb_fn: unsafe extern "C-unwind" fn(u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(*closure_ptr);
    for i in 0..(*arr).length as usize {
        let val = *(*arr).data.add(i);
        let idx = crate::circ::box_number(i as f64);
        let result = cb_fn(cb_tagged, 0xFFF1_0000_0000_0000, val, idx, arr_tagged);
        if is_truthy(result) {
            return 0xFFF4_0000_0000_0000; // true
        }
    }
    0xFFF3_0000_0000_0000 // false
}

/// `arr.reduce(fn, init)` — reduces to a single value.
/// Callback signature: fn(closure_env, accumulator, currentValue, index, array) -> accumulator
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_reduce(arr_tagged: u64, cb_tagged: u64, init: u64) -> u64 {
    let arr = untag_array(arr_tagged);
    if arr.is_null() { return init; }
    let mut acc = init;
    let closure_ptr = (cb_tagged & PAYLOAD_MASK) as *const u64;
    let cb_fn: unsafe extern "C-unwind" fn(u64, u64, u64, u64, u64, u64) -> u64 = std::mem::transmute(*closure_ptr);
    for i in 0..(*arr).length as usize {
        let val = *(*arr).data.add(i);
        let idx = crate::circ::box_number(i as f64);
        acc = cb_fn(cb_tagged, 0xFFF1_0000_0000_0000, acc, val, idx, arr_tagged);
    }
    acc
}

// ===========================================================================
// CIRC integration
// ===========================================================================

/// Free the element buffer when an array is destroyed via CIRC.
/// Called by the array's `drop_fn` (if one is registered).
pub unsafe fn free_array_data(arr_ptr: *mut u8) {
    let arr = arr_ptr as *mut BsArray;
    if !(*arr).data.is_null() {
        for i in 0..(*arr).length as usize {
            let val = *(*arr).data.add(i);
            if val != 0 && val != 0xFFF1_0000_0000_0000 {
                crate::circ::circ_dec_tagged(val);
            }
        }
        libc::free((*arr).data as *mut libc::c_void);
        (*arr).data = std::ptr::null_mut();
    }
}

/// Free ONLY the array element buffer without decrementing children (for cycle collector sweep)
pub unsafe fn free_array_buffer_only(arr_ptr: *mut u8) {
    let arr = arr_ptr as *mut BsArray;
    if !(*arr).data.is_null() {
        libc::free((*arr).data as *mut libc::c_void);
        (*arr).data = std::ptr::null_mut();
    }
}

/// Trace array elements for cycle collection
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_trace_elements(arr_ptr: *mut u8, visitor_ptr: *const ()) {
    let arr = arr_ptr as *mut BsArray;
    if (*arr).data.is_null() || (*arr).length == 0 { return; }
    
    let visitor: unsafe extern "C-unwind" fn(*mut crate::circ::CircHeader) = std::mem::transmute(visitor_ptr);
    for i in 0..(*arr).length as usize {
        let val = *(*arr).data.add(i);
        if crate::circ::is_managed_ptr(val) {
            let ptr = (val & 0x0000_FFFF_FFFF_FFFF) as *mut u8;
            if !ptr.is_null() {
                let header = ptr.sub(crate::circ::CircHeader::SIZE) as *mut crate::circ::CircHeader;
                visitor(header);
            }
        }
    }
}

// ===========================================================================
// Helpers
// ===========================================================================

/// Check if a NaN-boxed value is truthy (JS semantics).
fn is_truthy(val: u64) -> bool {
    if val == 0 { return false; } // undefined
    if val == 0xFFF3_0000_0000_0000 { return false; } // false
    if val == 0xFFF5_0000_0000_0000 { return false; } // null
    // Check for NaN
    let tag = val & TAG_MASK;
    if tag == 0 || (tag > 0 && tag < 0xFFF0_0000_0000_0000) {
        // It's a number
        let f = f64::from_bits(val);
        if f == 0.0 || f.is_nan() { return false; }
    }
    // Check for empty string
    if tag == 0xFFF7_0000_0000_0000 {
        let payload = val & PAYLOAD_MASK;
        if payload != 0 {
            let c_str = unsafe { std::ffi::CStr::from_ptr(payload as *const libc::c_char) };
            if c_str.to_bytes().is_empty() { return false; }
        }
    }
    true
}

/// Convert a NaN-boxed value to a display string (for arr.join, etc.).
unsafe fn value_to_string(val: u64) -> String {
    if val == 0 { return String::new(); } // undefined → ""
    let tag = val & TAG_MASK;
    if tag == 0xFFF3_0000_0000_0000 { return "false".to_string(); }
    if tag == 0xFFF4_0000_0000_0000 { return "true".to_string(); }
    if tag == 0xFFF5_0000_0000_0000 { return "null".to_string(); }
    if tag == 0xFFF7_0000_0000_0000 {
        let payload = val & PAYLOAD_MASK;
        if payload == 0 { return String::new(); }
        let c_str = unsafe { std::ffi::CStr::from_ptr(payload as *const libc::c_char) };
        return c_str.to_str().unwrap_or("").to_string();
    }
    // Number
    let f = f64::from_bits(val);
    if f == f.floor() && f.abs() < 1e15 {
        format!("{}", f as i64)
    } else {
        format!("{}", f)
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_alloc_array() -> u64 {
    __bs_array_new()
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_alloc_owned_array() -> u64 {
    let size = std::mem::size_of::<BsArray>();
    let total_size = crate::circ::CircHeader::SIZE + size;

    let (raw, alloc_size) = crate::slab::fast_alloc_shared(total_size);

    let header = raw as *mut crate::circ::CircHeader;
    (*header).local_rc = 0;
    (*header).global_rc.store(0, std::sync::atomic::Ordering::Relaxed);
    (*header).owner_tid.store(crate::circ::NO_OWNER, std::sync::atomic::Ordering::Relaxed);
    (*header).flags.store(crate::circ::IS_ARRAY, std::sync::atomic::Ordering::Relaxed);
    (*header).alloc_size = alloc_size;
    (*header).crc = 0;

    let ptr = (raw as *mut u8).add(crate::circ::CircHeader::SIZE);
    let arr = ptr as *mut BsArray;
    (*arr).length = 0;
    (*arr).capacity = 0;
    (*arr).data = std::ptr::null_mut();
    crate::verify::__bs_verify_track_alloc(ptr);
    
    // Increment owned stats
    crate::circ::OWNED_ALLOCS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    
    (ptr as u64) | crate::dynamic_call::helpers::TAG_OWNED_ARRAY
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_alloc_arena_array(arena: *mut crate::arena::Arena) -> u64 {
    let size = std::mem::size_of::<BsArray>();
    let total_size = crate::circ::CircHeader::SIZE + size;

    let raw = crate::arena::arena_alloc(arena, total_size, 8);

    let header = raw as *mut crate::circ::CircHeader;
    (*header).local_rc = 0;
    (*header).global_rc.store(0, std::sync::atomic::Ordering::Relaxed);
    (*header).owner_tid.store(crate::circ::NO_OWNER, std::sync::atomic::Ordering::Relaxed);
    (*header).flags.store(crate::circ::IS_ARRAY, std::sync::atomic::Ordering::Relaxed);
    (*header).alloc_size = total_size as u16;
    (*header).crc = 0;

    let ptr = (raw as *mut u8).add(crate::circ::CircHeader::SIZE);
    let arr = ptr as *mut BsArray;
    (*arr).length = 0;
    (*arr).capacity = 0;
    (*arr).data = std::ptr::null_mut();
    
    // Register destructor to free the internal dynamic buffer
    crate::arena::arena_register_dtor(arena, ptr, drop_arena_array);
    
    // We do NOT track arena allocations in the global verifier
    // crate::verify::__bs_verify_track_alloc(ptr);
    
    (ptr as u64) | crate::dynamic_call::helpers::TAG_ARENA_ARRAY
}

unsafe extern "C-unwind" fn drop_arena_array(ptr: *mut u8) {
    let arr = ptr as *mut BsArray;
    if !(*arr).data.is_null() {
        libc::free((*arr).data as *mut libc::c_void);
        (*arr).data = std::ptr::null_mut();
    }
}

/// `arr.at(index)` — get element at relative index.
#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_array_at(arr_tagged: u64, index_tagged: u64) -> u64 {
    let arr = untag_array(arr_tagged);
    if arr.is_null() { return 0xFFF1_0000_0000_0000; }
    let mut idx = f64::from_bits(index_tagged) as i64;
    let len = (*arr).length as i64;
    if idx < 0 {
        idx += len;
    }
    if idx < 0 || idx >= len {
        return 0xFFF1_0000_0000_0000; // undefined
    }
    let val = *(*arr).data.add(idx as usize);
    crate::circ::circ_inc_tagged(val);
    val
}
