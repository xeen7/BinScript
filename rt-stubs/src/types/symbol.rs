use std::sync::atomic::{AtomicU64, Ordering};
use crate::types::coercion::__bs_String;
use crate::types::string_utils::create_tagged_string;
use crate::objects::builtins::__bs_new_object;
use crate::objects::dynamic_props::set_dynamic_property;

/// Global counter to ensure each Symbol() call produces a unique value.
/// The counter is stored in the upper bits of the payload to differentiate
/// symbols with the same description.
static SYMBOL_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Creates a new Symbol primitive.
/// The description string pointer is stored as the payload.
/// Each symbol is unique (tracked via SYMBOL_COUNTER).
#[no_mangle]
pub unsafe extern "C" fn __bs_Symbol(desc: u64) -> u64 {
    let _unique_id = SYMBOL_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tag = desc & 0xFFFF_0000_0000_0000;
    let desc_ptr = if tag == 0xFFF7_0000_0000_0000 {
        // Extract string pointer payload
        (desc & 0x0000_FFFF_FFFF_FFFF) as *const u8
    } else if tag == 0xFFF1_0000_0000_0000 {
        // undefined -> no description
        std::ptr::null()
    } else {
        // Coerce to string first
        let s_tagged = __bs_String(desc);
        (s_tagged & 0x0000_FFFF_FFFF_FFFF) as *const u8
    };

    // Allocate a small block to store (unique_id, desc_ptr) so each symbol is distinct
    let block = libc::malloc(16) as *mut u64;
    *block = _unique_id;
    *(block.add(1)) = desc_ptr as u64;

    (block as u64) | 0xFFF8_0000_0000_0000
}

/// No-arg Symbol() call
#[no_mangle]
pub unsafe extern "C" fn __bs_Symbol_0() -> u64 {
    __bs_Symbol(0xFFF1_0000_0000_0000) // undefined
}

/// 1-arg Symbol(desc) call
#[no_mangle]
pub unsafe extern "C" fn __bs_Symbol_1(desc: u64) -> u64 {
    __bs_Symbol(desc)
}

/// Returns a global object with well-known symbol properties.
#[no_mangle]
pub unsafe extern "C" fn __bs_get_Symbol_global() -> u64 {
    let obj = __bs_new_object();
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    let obj_ptr = payload as *mut u8;

    // Create well-known symbols
    let well_known = [
        "iterator",
        "asyncIterator",
        "hasInstance",
        "toPrimitive",
        "toStringTag",
    ];
    for name in &well_known {
        let desc_tagged = create_tagged_string(&format!("Symbol.{}", name));
        let sym = __bs_Symbol(desc_tagged);
        set_dynamic_property(obj_ptr, name.to_string(), sym);
    }

    obj
}
