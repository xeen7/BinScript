use std::sync::{Arc, Mutex};
use sonic_rs::JsonValueTrait;

pub const TAG_JSON_TAPE: u64 = 0xFFF8_0000_0000_0000;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TapeState {
    Raw,
    Indexed,
    Parsed,
}

pub struct JsonTape {
    pub raw: Arc<[u8]>,
    pub tape: Option<sonic_rs::Value>,
    pub state: TapeState,
}

#[no_mangle]
pub unsafe extern "C" fn __bs_json_parse_lazy(ptr: *const u8, len: u32) -> u64 {
    let slice = std::slice::from_raw_parts(ptr, len as usize);
    let raw = Arc::from(slice);

    let tape = Box::new(Mutex::new(JsonTape {
        raw,
        tape: None,
        state: TapeState::Raw,
    }));
    let boxed_ptr = Box::into_raw(tape);
    
    // Register to global roots to prevent the JSON tape itself from being collected
    let tagged = (boxed_ptr as u64) | TAG_JSON_TAPE;
    crate::gc::GLOBAL_ROOTS.lock().unwrap().push(tagged);
    tagged
}

#[no_mangle]
pub unsafe extern "C" fn __bs_json_tape_get(tape_tagged: u64, key_ptr: *const u8, key_len: u32) -> u64 {
    let tag = tape_tagged & 0xFFFF_0000_0000_0000;
    if tag != TAG_JSON_TAPE {
        return 0xFFF1_0000_0000_0000; // undefined
    }

    let ptr = (tape_tagged & 0x0000_FFFF_FFFF_FFFF) as *mut Mutex<JsonTape>;
    let mut tape_obj = (*ptr).lock().unwrap();

    let key_slice = std::slice::from_raw_parts(key_ptr, key_len as usize);
    let key_str = std::str::from_utf8_unchecked(key_slice);

    if tape_obj.state == TapeState::Raw {
        // Parse it lazily
        let raw_str = std::str::from_utf8_unchecked(&tape_obj.raw);
        if let Ok(value) = sonic_rs::from_str::<sonic_rs::Value>(raw_str) {
            tape_obj.tape = Some(value);
            tape_obj.state = TapeState::Indexed;
        } else {
            return 0xFFF1_0000_0000_0000; // Return undefined on parse error
        }
    }

    if let Some(val) = &tape_obj.tape {
        if let Some(field) = val.get(key_str) {
            if field.is_number() {
                let num = field.as_f64().unwrap_or(0.0);
                return crate::gc::box_number(num);
            } else if field.is_str() {
                // Return a string pointer. For now we just return undefined to keep it simple,
                // since we haven't implemented full string GC handling. 
                // But for tests, we might want numbers mostly.
                // Wait, we don't have a string representation implemented yet!
                return 0xFFF1_0000_0000_0000;
            } else if field.is_boolean() {
                if field.as_bool().unwrap() {
                    return 0xFFF4_0000_0000_0000; // true
                } else {
                    return 0xFFF3_0000_0000_0000; // false
                }
            } else if field.is_null() {
                return 0xFFF2_0000_0000_0000; // null
            }
        }
    }

    0xFFF1_0000_0000_0000 // undefined
}

#[no_mangle]
pub unsafe extern "C" fn __bs_json_tape_materialize(tape_tagged: u64) -> u64 {
    // For now, this just returns the tape tag unmodified as we don't have a full object
    // allocator to materialize the tree into a JsObject representation yet.
    tape_tagged
}

#[no_mangle]
pub unsafe extern "C" fn __bs_prop_get(obj_tagged: u64, prop_str: *const u8, len: u32) -> u64 {
    let tag = obj_tagged & 0xFFFF_0000_0000_0000;
    if tag == TAG_JSON_TAPE {
        return __bs_json_tape_get(obj_tagged, prop_str, len);
    }
    if tag == 0xFFFB_0000_0000_0000 {
        let prop_slice = std::slice::from_raw_parts(prop_str, len as usize);
        if prop_slice == b"length" {
            return crate::array::__bs_array_length(obj_tagged);
        }
        if let Ok(s) = std::str::from_utf8(prop_slice) {
            if let Ok(idx) = s.parse::<f64>() {
                let idx_boxed = crate::gc::box_number(idx);
                return crate::array::__bs_array_get(obj_tagged, idx_boxed);
            }
        }
        return 0xFFF1_0000_0000_0000; // undefined
    }
    if tag == 0xFFF7_0000_0000_0000 {
        let prop_slice = std::slice::from_raw_parts(prop_str, len as usize);
        if prop_slice == b"length" {
            return crate::string_methods::__bs_string_length(obj_tagged);
        }
        return 0xFFF1_0000_0000_0000; // undefined
    }
    if tag != 0xFFF6_0000_0000_0000 {
        return 0xFFF1_0000_0000_0000; // undefined
    }
    
    let payload = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    if payload == 0 {
        return 0xFFF1_0000_0000_0000; // undefined
    }

    let prop_slice = std::slice::from_raw_parts(prop_str, len as usize);
    let obj_ptr = payload as *mut u8;
    
    // 1. Check class fields if vtable is present
    let vtable_ptr = *(obj_ptr as *const *const crate::VTable);
    if !vtable_ptr.is_null() {
        let vtable = &*vtable_ptr;
        let fields_count = vtable.fields_count as usize;
        if fields_count > 0 && !vtable.field_names.is_null() {
            for i in 0..fields_count {
                let name_ptr = *vtable.field_names.add(i);
                if !name_ptr.is_null() {
                    let name_cstr = std::ffi::CStr::from_ptr(name_ptr as *const libc::c_char);
                    let name_bytes = name_cstr.to_bytes();
                    if name_bytes == prop_slice {
                        let slot_ptr = (obj_ptr as *const u64).add(1 + i);
                        return *slot_ptr;
                    }
                }
            }
        }
    }

    // 2. Check dynamic properties
    if let Ok(prop_name) = std::str::from_utf8(prop_slice) {
        if let Some(val) = crate::get_dynamic_property(obj_ptr, prop_name) {
            return val;
        }
    }

    // 3. Walk the __proto__ chain
    let mut current_obj = obj_tagged;
    loop {
        let current_payload = current_obj & 0x0000_FFFF_FFFF_FFFF;
        if current_payload == 0 {
            break;
        }
        let current_ptr = current_payload as *mut u8;
        if let Some(proto) = crate::get_dynamic_property(current_ptr, "__proto__") {
            let proto_tag = proto & 0xFFFF_0000_0000_0000;
            if proto_tag == 0xFFF6_0000_0000_0000 {
                let proto_payload = proto & 0x0000_FFFF_FFFF_FFFF;
                let proto_ptr = proto_payload as *mut u8;
                // Check class fields of the prototype first
                let proto_vtable_ptr = *(proto_ptr as *const *const crate::VTable);
                if !proto_vtable_ptr.is_null() {
                    let vtable = &*proto_vtable_ptr;
                    let fields_count = vtable.fields_count as usize;
                    if fields_count > 0 && !vtable.field_names.is_null() {
                        for i in 0..fields_count {
                            let name_ptr = *vtable.field_names.add(i);
                            if !name_ptr.is_null() {
                                let name_cstr = std::ffi::CStr::from_ptr(name_ptr as *const libc::c_char);
                                let name_bytes = name_cstr.to_bytes();
                                if name_bytes == prop_slice {
                                    let slot_ptr = (proto_ptr as *const u64).add(1 + i);
                                    return *slot_ptr;
                                }
                            }
                        }
                    }
                }
                // Check dynamic properties of the prototype
                if let Ok(prop_name) = std::str::from_utf8(prop_slice) {
                    if let Some(val) = crate::get_dynamic_property(proto_ptr, prop_name) {
                        return val;
                    }
                }
                current_obj = proto;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    0xFFF1_0000_0000_0000 // undefined
}

#[no_mangle]
pub unsafe extern "C" fn __bs_prop_set(obj_tagged: u64, prop_str: *const u8, len: u32, val_tagged: u64) {
    let tag = obj_tagged & 0xFFFF_0000_0000_0000;
    if tag == TAG_JSON_TAPE {
        return;
    }
    if tag == 0xFFFB_0000_0000_0000 {
        let prop_slice = std::slice::from_raw_parts(prop_str, len as usize);
        if let Ok(s) = std::str::from_utf8(prop_slice) {
            if let Ok(idx) = s.parse::<f64>() {
                let idx_boxed = crate::gc::box_number(idx);
                crate::array::__bs_array_set(obj_tagged, idx_boxed, val_tagged);
            }
        }
        return;
    }
    if tag != 0xFFF6_0000_0000_0000 {
        return;
    }
    
    let payload = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    if payload == 0 {
        return;
    }

    let prop_slice = std::slice::from_raw_parts(prop_str, len as usize);
    let obj_ptr = payload as *mut u8;
    
    // 1. Check class fields if vtable is present
    let vtable_ptr = *(obj_ptr as *const *const crate::VTable);
    if !vtable_ptr.is_null() {
        let vtable = &*vtable_ptr;
        let fields_count = vtable.fields_count as usize;
        if fields_count > 0 && !vtable.field_names.is_null() {
            for i in 0..fields_count {
                let name_ptr = *vtable.field_names.add(i);
                if !name_ptr.is_null() {
                    let name_cstr = std::ffi::CStr::from_ptr(name_ptr as *const libc::c_char);
                    let name_bytes = name_cstr.to_bytes();
                    if name_bytes == prop_slice {
                        let slot_mut_ptr = (obj_ptr as *mut u64).add(1 + i);
                        *slot_mut_ptr = val_tagged;
                        return;
                    }
                }
            }
        }
    }

    // 2. Otherwise set as dynamic property
    if let Ok(prop_name) = std::str::from_utf8(prop_slice) {
        crate::set_dynamic_property(obj_ptr, prop_name.to_string(), val_tagged);
    }
}
