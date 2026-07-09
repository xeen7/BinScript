use crate::VTable;

pub const TAG_ARRAY: u64 = 0xFFFB_0000_0000_0000;
pub const TAG_STRING: u64 = 0xFFF7_0000_0000_0000;
pub const TAG_OBJECT: u64 = 0xFFF6_0000_0000_0000;
pub const TAG_OWNED: u64 = 0xFFFC_0000_0000_0000;
pub const TAG_OWNED_CLOSURE: u64 = 0x7FF9_0000_0000_0000;
pub const TAG_OWNED_ARRAY: u64 = 0x7FFB_0000_0000_0000;
pub const TAG_OWNED_STRING: u64 = 0x7FF7_0000_0000_0000;
pub const TAG_ARENA_ARRAY: u64 = 0x7FFA_0000_0000_0000;
pub const TAG_ARENA: u64 = 0xFFFE_0000_0000_0000;
pub const TAG_MASK: u64 = 0xFFFF_0000_0000_0000;
pub const PAYLOAD_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;

#[inline(always)]
pub fn is_array_tag(tag: u64) -> bool {
    let t = tag & TAG_MASK;
    t == TAG_ARRAY || t == TAG_OWNED_ARRAY || t == TAG_ARENA_ARRAY
}

pub unsafe fn get_user_method(receiver: u64, idx: i32) -> Option<*const u8> {
    if idx < 0 { return None; }
    let payload = receiver & PAYLOAD_MASK;
    if payload == 0 { return None; }
    let obj_ptr = payload as *const *const VTable;
    let mut vtable = *obj_ptr;
    while !vtable.is_null() {
        let slot = *(vtable as *const *const u8).add(5 + idx as usize);
        if !slot.is_null() {
            return Some(slot);
        }
        vtable = (*vtable).parent;
    }
    None
}

/// Convert a NaN-boxed value to a display string.
pub unsafe fn value_to_string(val: u64) -> String {
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


#[inline(always)]
pub fn is_number_tag(tag: u64) -> bool {
    let tag = tag & 0xFFFF_0000_0000_0000;
    if tag == 0x7FF7_0000_0000_0000 || tag == 0x7FF9_0000_0000_0000 || tag == 0x7FFA_0000_0000_0000 || tag == 0x7FFB_0000_0000_0000 {
        return false;
    }
    tag < 0xFFF0_0000_0000_0000
}
