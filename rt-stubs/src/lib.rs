//! Runtime stubs for BinScript compiled binaries.
//!
//! In Stage 2, runtime helpers contain memory allocation, zero-initialization,
//! and prototype inheritance verification stubs.

pub mod gc;
pub mod shadow_stack;
pub mod microtask;
pub mod promise;
pub mod promise_combinators;
pub mod json_tape;
pub mod array;
pub mod string_methods;
pub mod dynamic_call;
pub mod math_global;
pub mod exception;

use crate::array::__bs_array_new;
use sonic_rs::{JsonValueTrait, JsonContainerTrait};

#[repr(C)]
pub struct VTable {
    pub parent: *const VTable,
    pub name: *const u8,
    pub shape_id: u64,
    pub fields_count: u64,
    pub field_names: *const *const u8,
}

unsafe impl Sync for VTable {}
unsafe impl Send for VTable {}

/// Allocates memory for a class object of size_in_bytes, zero-initializes it,
/// sets the vtable pointer at offset 0, and returns the tagged NaN-boxed pointer.
///
/// NaN-boxing tag for object: TAG_OBJECT = 0xFFF6_0000_0000_0000.
#[no_mangle]
pub unsafe extern "C" fn __bs_alloc(vtable_ptr: *const VTable, size_in_bytes: usize) -> u64 {
    // Objects have their first 8 bytes as the vtable.
    // The rest of the object may contain GC pointers. We'll conservatively scan everything.
    // Assuming size_in_bytes is a multiple of 8.
    let num_slots = (size_in_bytes / 8) as u16;
    let ptr = gc::gc_alloc(size_in_bytes, 0xFFF6, num_slots);
    
    // Store vtable pointer at offset 0
    let obj_ptr = ptr as *mut *const VTable;
    *obj_ptr = vtable_ptr;
    
    // Return as NaN-boxed Object pointer
    (ptr as u64) | 0xFFF6_0000_0000_0000
}

/// Traverses the prototype chain of an object to verify if it inherits from a target shape ID.
///
/// Returns TAG_TRUE (0xFFF4_0000_0000_0000) if a match is found,
/// otherwise TAG_FALSE (0xFFF3_0000_0000_0000).
#[no_mangle]
pub unsafe extern "C" fn __bs_instanceof(obj_val: u64, target_shape_id: u64) -> u64 {
    // target_shape_id is passed as a NaN-boxed float, so we convert its bits back to f64 and cast to u64
    let target_shape_id_f64 = f64::from_bits(target_shape_id);
    let target_shape_id_u64 = target_shape_id_f64 as u64;

    // Check if the NaN tag is exactly TAG_OBJECT (0xFFF6)
    let tag = obj_val & 0xFFFF_0000_0000_0000;
    if tag != 0xFFF6_0000_0000_0000 {
        return 0xFFF3_0000_0000_0000; // TAG_FALSE
    }
    // Extract the raw pointer (payload)
    let payload = obj_val & 0x0000_FFFF_FFFF_FFFF;
    if payload == 0 {
        return 0xFFF3_0000_0000_0000; // TAG_FALSE
    }
    let obj_ptr = payload as *const *const VTable;
    let mut vtable = *obj_ptr;
    // Traverse parent hierarchy in the prototype chain
    while !vtable.is_null() {
        if (*vtable).shape_id == target_shape_id_u64 {
            return 0xFFF4_0000_0000_0000; // TAG_TRUE
        }
        vtable = (*vtable).parent;
    }
    0xFFF3_0000_0000_0000 // TAG_FALSE
}

/// Allocates memory for a closure env of size_in_bytes, zero-initializes it,
/// and returns the tagged NaN-boxed closure pointer.
///
/// NaN-boxing tag for closure: TAG_CLOSURE = 0xFFF9_0000_0000_0000.
#[no_mangle]
pub unsafe extern "C" fn __bs_alloc_closure(size_in_bytes: usize) -> u64 {
    let num_slots = (size_in_bytes / 8) as u16;
    let ptr = gc::gc_alloc(size_in_bytes, 0xFFF9, num_slots);
    (ptr as u64) | 0xFFF9_0000_0000_0000
}

/// Allocates memory for a generator state struct of size_in_bytes, zero-initializes it,
/// and returns the tagged NaN-boxed generator pointer.
///
/// NaN-boxing tag for generator: TAG_GENERATOR = 0xFFFA_0000_0000_0000.
#[no_mangle]
pub unsafe extern "C" fn __bs_alloc_generator(size_in_bytes: usize) -> u64 {
    let num_slots = (size_in_bytes / 8) as u16;
    let ptr = gc::gc_alloc(size_in_bytes, 0xFFFA, num_slots);
    (ptr as u64) | 0xFFFA_0000_0000_0000
}

#[repr(C)]
pub struct GeneratorState {
    pub state_idx: i64,
    pub poll_fn: extern "C" fn(*mut GeneratorState, u64) -> u64,
}

#[no_mangle]
pub unsafe extern "C" fn __bs_generator_next(gen_tagged: u64, sent_value: u64) -> u64 {
    let tag = gen_tagged & 0xFFFF_0000_0000_0000;
    if tag != 0xFFFA_0000_0000_0000 {
        panic!("__bs_generator_next called on non-generator");
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
pub unsafe extern "C" fn __bs_generator_is_done(gen_tagged: u64) -> u64 {
    let tag = gen_tagged & 0xFFFF_0000_0000_0000;
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

pub fn stub_version() -> &'static str {
    "0.3.0-stage3"
}

// Helper to extract a Rust &str from a NaN-boxed string pointer
pub unsafe fn get_c_string_from_tagged(val: u64) -> &'static str {
    let tag = val & 0xFFFF_0000_0000_0000;
    if tag != 0xFFF7_0000_0000_0000 {
        panic!("Expected string value");
    }
    let payload = val & 0x0000_FFFF_FFFF_FFFF;
    let c_str = std::ffi::CStr::from_ptr(payload as *const libc::c_char);
    c_str.to_str().expect("Invalid UTF-8 string")
}

// Helper to allocate a null-terminated string using malloc and return it boxed
pub unsafe fn create_tagged_string(s: &str) -> u64 {
    let len = s.len();
    let ptr = libc::malloc(len + 1) as *mut u8;
    std::ptr::copy_nonoverlapping(s.as_ptr(), ptr, len);
    *ptr.add(len) = 0; // null terminator
    (ptr as u64) | 0xFFF7_0000_0000_0000
}

#[no_mangle]
pub unsafe extern "C" fn __bs_fs_read_file_sync(path_tagged: u64) -> u64 {
    let path_str = get_c_string_from_tagged(path_tagged);
    match std::fs::read_to_string(path_str) {
        Ok(content) => create_tagged_string(&content),
        Err(_) => create_tagged_string(""),
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_fs_write_file_sync(path_tagged: u64, data_tagged: u64) {
    let path_str = get_c_string_from_tagged(path_tagged);
    let data_str = get_c_string_from_tagged(data_tagged);
    let _ = std::fs::write(path_str, data_str);
}

#[no_mangle]
pub unsafe extern "C" fn __bs_fs_exists_sync(path_tagged: u64) -> u64 {
    let path_str = get_c_string_from_tagged(path_tagged);
    if std::path::Path::new(path_str).exists() {
        0xFFF4_0000_0000_0000 // TAG_TRUE
    } else {
        0xFFF3_0000_0000_0000 // TAG_FALSE
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_path_join(a_tagged: u64, b_tagged: u64) -> u64 {
    let a_str = get_c_string_from_tagged(a_tagged);
    let b_str = get_c_string_from_tagged(b_tagged);
    let path = std::path::PathBuf::from(a_str).join(b_str);
    create_tagged_string(&path.to_string_lossy())
}

#[no_mangle]
pub unsafe extern "C" fn __bs_path_resolve(a_tagged: u64, b_tagged: u64) -> u64 {
    let a_str = get_c_string_from_tagged(a_tagged);
    let b_str = get_c_string_from_tagged(b_tagged);
    let joined = std::path::Path::new(a_str).join(b_str);
    create_tagged_string(&joined.to_string_lossy())
}

#[no_mangle]
pub unsafe extern "C" fn __bs_os_platform() -> u64 {
    create_tagged_string("linux")
}

#[no_mangle]
pub unsafe extern "C" fn __bs_os_arch() -> u64 {
    create_tagged_string("x64")
}

use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub static DYNAMIC_PROPERTIES: Lazy<Mutex<HashMap<usize, HashMap<String, u64>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

#[no_mangle]
pub unsafe extern "C" fn __bs_new_object() -> u64 {
    __bs_alloc(&OBJECT_VTABLE, 8)
}

#[no_mangle]
pub unsafe extern "C" fn __bs_object_spread(target_tagged: u64, source_tagged: u64) -> u64 {
    let target_tag = target_tagged & 0xFFFF_0000_0000_0000;
    if target_tag != 0xFFF6_0000_0000_0000 {
        return target_tagged;
    }
    let target_payload = target_tagged & 0x0000_FFFF_FFFF_FFFF;
    if target_payload == 0 {
        return target_tagged;
    }
    let target_ptr = target_payload as *mut u8;
    
    let source_tag = source_tagged & 0xFFFF_0000_0000_0000;
    if source_tag == 0xFFF6_0000_0000_0000 {
        let src_payload = source_tagged & 0x0000_FFFF_FFFF_FFFF;
        if src_payload != 0 {
            let src_ptr = src_payload as *mut u8;
            
            // 1. Copy class fields if vtable is present in source
            let vtable_ptr = *(src_ptr as *const *const VTable);
            if !vtable_ptr.is_null() {
                let vtable = &*vtable_ptr;
                let fields_count = vtable.fields_count as usize;
                if fields_count > 0 && !vtable.field_names.is_null() {
                    for i in 0..fields_count {
                        let name_ptr = *vtable.field_names.add(i);
                        if !name_ptr.is_null() {
                            let name_cstr = std::ffi::CStr::from_ptr(name_ptr as *const libc::c_char);
                            if let Ok(name_str) = name_cstr.to_str() {
                                let val = *(src_ptr as *const u64).add(1 + i);
                                crate::set_dynamic_property(target_ptr, name_str.to_string(), val);
                            }
                        }
                    }
                }
            }
            
            // 2. Copy dynamic properties of source
            let props: Vec<(String, u64)> = {
                let map = DYNAMIC_PROPERTIES.lock().unwrap();
                if let Some(obj_entry) = map.get(&(src_payload as usize)) {
                    obj_entry.iter().map(|(k, &v)| (k.clone(), v)).collect()
                } else {
                    Vec::new()
                }
            };
            for (k, v) in props {
                if k != "__proto__" {
                    crate::set_dynamic_property(target_ptr, k, v);
                }
            }
        }
    } else if source_tag == 0xFFF8_0000_0000_0000 { // TAG_JSON_TAPE
        let src_payload = source_tagged & 0x0000_FFFF_FFFF_FFFF;
        if src_payload != 0 {
            let ptr = src_payload as *mut std::sync::Mutex<crate::json_tape::JsonTape>;
            let mut tape_obj = (*ptr).lock().unwrap();
            
            if tape_obj.state == crate::json_tape::TapeState::Raw {
                let raw_str = std::str::from_utf8_unchecked(&tape_obj.raw);
                if let Ok(value) = sonic_rs::from_str::<sonic_rs::Value>(raw_str) {
                    tape_obj.tape = Some(value);
                    tape_obj.state = crate::json_tape::TapeState::Indexed;
                }
            }
            
            if let Some(val) = &tape_obj.tape {
                if let Some(obj) = val.as_object() {
                    for (k, v) in obj.iter() {
                        let val_tagged = sonic_value_to_tagged(v);
                        crate::set_dynamic_property(target_ptr, k.to_string(), val_tagged);
                    }
                }
            }
        }
    }
    
    target_tagged
}

unsafe fn sonic_value_to_tagged(field: &sonic_rs::Value) -> u64 {
    if field.is_number() {
        let num = field.as_f64().unwrap_or(0.0);
        crate::gc::box_number(num)
    } else if field.is_str() {
        if let Some(s) = field.as_str() {
            crate::create_tagged_string(s)
        } else {
            0xFFF1_0000_0000_0000
        }
    } else if field.is_boolean() {
        if field.as_bool().unwrap_or(false) {
            0xFFF4_0000_0000_0000
        } else {
            0xFFF3_0000_0000_0000
        }
    } else if field.is_null() {
        0xFFF2_0000_0000_0000
    } else {
        0xFFF1_0000_0000_0000 // undefined
    }
}

pub unsafe fn set_dynamic_property(obj_ptr: *mut u8, prop_name: String, val_tagged: u64) {
    let mut map = DYNAMIC_PROPERTIES.lock().unwrap();
    let obj_entry = map.entry(obj_ptr as usize).or_insert_with(HashMap::new);
    obj_entry.insert(prop_name, val_tagged);
}

pub unsafe fn get_dynamic_property(obj_ptr: *mut u8, prop_name: &str) -> Option<u64> {
    let map = DYNAMIC_PROPERTIES.lock().unwrap();
    if let Some(obj_entry) = map.get(&(obj_ptr as usize)) {
        return obj_entry.get(prop_name).copied();
    }
    None
}

pub unsafe fn delete_dynamic_property(obj_ptr: *mut u8, prop_name: &str) -> bool {
    let mut map = DYNAMIC_PROPERTIES.lock().unwrap();
    if let Some(obj_entry) = map.get_mut(&(obj_ptr as usize)) {
        return obj_entry.remove(prop_name).is_some();
    }
    false
}

pub unsafe fn remove_dynamic_properties(obj_ptr: *mut u8) {
    let mut map = DYNAMIC_PROPERTIES.lock().unwrap();
    map.remove(&(obj_ptr as usize));
}

pub unsafe fn trace_dynamic_properties(obj_ptr: *mut u8) {
    let vals: Vec<u64> = {
        let map = DYNAMIC_PROPERTIES.lock().unwrap();
        if let Some(obj_entry) = map.get(&(obj_ptr as usize)) {
            obj_entry.values().copied().collect()
        } else {
            Vec::new()
        }
    };
    for val in vals {
        gc::gc_mark_value(val);
    }
}

// ===========================================================================
// Builtin Objects VTables
// ===========================================================================

#[no_mangle]
pub static OBJECT_VTABLE: VTable = VTable {
    parent: std::ptr::null(),
    name: b"Object\0".as_ptr(),
    shape_id: 1001,
    fields_count: 0,
    field_names: std::ptr::null(),
};

#[no_mangle]
pub static STRING_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"String\0".as_ptr(),
    shape_id: 1002,
    fields_count: 0,
    field_names: std::ptr::null(),
};

#[no_mangle]
pub static NUMBER_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"Number\0".as_ptr(),
    shape_id: 1003,
    fields_count: 0,
    field_names: std::ptr::null(),
};

#[no_mangle]
pub static BOOLEAN_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"Boolean\0".as_ptr(),
    shape_id: 1004,
    fields_count: 0,
    field_names: std::ptr::null(),
};

#[no_mangle]
pub static DATE_VTABLE: VTable = VTable {
    parent: &OBJECT_VTABLE,
    name: b"Date\0".as_ptr(),
    shape_id: 1005,
    fields_count: 0,
    field_names: std::ptr::null(),
};

// ===========================================================================
// typeof support
// ===========================================================================

#[no_mangle]
pub unsafe extern "C" fn __bs_typeof(val: u64) -> u64 {
    let tag = val & 0xFFFF_0000_0000_0000;
    let s = if tag == 0xFFF1_0000_0000_0000 {
        "undefined"
    } else if tag == 0xFFF2_0000_0000_0000 {
        "object" // null
    } else if tag == 0xFFF3_0000_0000_0000 || tag == 0xFFF4_0000_0000_0000 {
        "boolean"
    } else if tag == 0xFFF7_0000_0000_0000 {
        "string"
    } else if tag == 0xFFF8_0000_0000_0000 {
        "symbol"
    } else if tag == 0xFFF9_0000_0000_0000 {
        "function" // closure
    } else if tag == 0xFFFA_0000_0000_0000 {
        "object" // generator
    } else if tag == 0xFFFB_0000_0000_0000 {
        "object" // array
    } else if tag == 0xFFFC_0000_0000_0000 {
        "object" // promise
    } else if tag == 0xFFF6_0000_0000_0000 {
        // object or class instance
        "object"
    } else {
        "number"
    };
    create_tagged_string(s)
}

// ===========================================================================
// Standard Coercions
// ===========================================================================

#[no_mangle]
pub unsafe extern "C" fn __bs_String(val: u64) -> u64 {
    let tag = val & 0xFFFF_0000_0000_0000;
    if tag == 0xFFF1_0000_0000_0000 {
        create_tagged_string("undefined")
    } else if tag == 0xFFF2_0000_0000_0000 {
        create_tagged_string("null")
    } else if tag == 0xFFF3_0000_0000_0000 {
        create_tagged_string("false")
    } else if tag == 0xFFF4_0000_0000_0000 {
        create_tagged_string("true")
    } else if tag == 0xFFF7_0000_0000_0000 {
        val
    } else if tag == 0xFFF8_0000_0000_0000 {
        // Symbol -> "Symbol(description)" or "Symbol()"
        let payload = val & 0x0000_FFFF_FFFF_FFFF;
        let block = payload as *const u64;
        let desc_ptr = *(block.add(1)) as *const u8;
        if desc_ptr.is_null() {
            create_tagged_string("Symbol()")
        } else {
            let c_str = std::ffi::CStr::from_ptr(desc_ptr as *const libc::c_char);
            let desc = c_str.to_str().unwrap_or("");
            create_tagged_string(&format!("Symbol({})", desc))
        }
    } else if tag == 0xFFF6_0000_0000_0000 {
        let payload = val & 0x0000_FFFF_FFFF_FFFF;
        let obj_ptr = payload as *mut u8;
        let vtable_ptr = *(obj_ptr as *const *const VTable);
        if !vtable_ptr.is_null() {
            let name_cstr = std::ffi::CStr::from_ptr((*vtable_ptr).name as *const libc::c_char);
            let name_bytes = name_cstr.to_bytes();
            if name_bytes == b"String" || name_bytes == b"Number" || name_bytes == b"Boolean" || name_bytes == b"Date" {
                if let Some(prim) = get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                    return __bs_String(prim);
                }
            }
        }
        create_tagged_string("[object Object]")
    } else if tag == 0xFFFB_0000_0000_0000 {
        let len_boxed = crate::array::__bs_array_length(val);
        let len = f64::from_bits(len_boxed) as usize;
        let mut parts = Vec::new();
        for i in 0..len {
            let elem = crate::array::__bs_array_get(val, gc::box_number(i as f64));
            let s_elem_tagged = __bs_String(elem);
            let s_elem = get_c_string_from_tagged(s_elem_tagged);
            parts.push(s_elem.to_string());
        }
        create_tagged_string(&parts.join(","))
    } else {
        let f = f64::from_bits(val);
        let s = if f.is_nan() {
            "NaN".to_string()
        } else if f.is_infinite() {
            if f.is_sign_positive() { "Infinity".to_string() } else { "-Infinity".to_string() }
        } else if f == f.floor() && f.abs() < 1e15 {
            format!("{}", f as i64)
        } else {
            format!("{}", f)
        };
        create_tagged_string(&s)
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Number(val: u64) -> u64 {
    let tag = val & 0xFFFF_0000_0000_0000;
    let num = if tag == 0xFFF1_0000_0000_0000 {
        f64::NAN
    } else if tag == 0xFFF2_0000_0000_0000 {
        0.0
    } else if tag == 0xFFF3_0000_0000_0000 {
        0.0
    } else if tag == 0xFFF4_0000_0000_0000 {
        1.0
    } else if tag == 0xFFF7_0000_0000_0000 {
        let s = get_c_string_from_tagged(val);
        s.trim().parse::<f64>().unwrap_or(f64::NAN)
    } else if tag == 0xFFF6_0000_0000_0000 {
        let payload = val & 0x0000_FFFF_FFFF_FFFF;
        let obj_ptr = payload as *mut u8;
        let vtable_ptr = *(obj_ptr as *const *const VTable);
        if !vtable_ptr.is_null() {
            let name_cstr = std::ffi::CStr::from_ptr((*vtable_ptr).name as *const libc::c_char);
            let name_bytes = name_cstr.to_bytes();
            if name_bytes == b"String" || name_bytes == b"Number" || name_bytes == b"Boolean" || name_bytes == b"Date" {
                if let Some(prim) = get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                    return __bs_Number(prim);
                }
            }
        }
        f64::NAN
    } else if tag == 0xFFFB_0000_0000_0000 {
        let s_tagged = __bs_String(val);
        let s = get_c_string_from_tagged(s_tagged);
        s.trim().parse::<f64>().unwrap_or(f64::NAN)
    } else {
        return val;
    };
    gc::box_number(num)
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Boolean(val: u64) -> u64 {
    let tag = val & 0xFFFF_0000_0000_0000;
    let b = if tag == 0xFFF1_0000_0000_0000 {
        false
    } else if tag == 0xFFF2_0000_0000_0000 {
        false
    } else if tag == 0xFFF3_0000_0000_0000 {
        false
    } else if tag == 0xFFF4_0000_0000_0000 {
        true
    } else if tag == 0xFFF7_0000_0000_0000 {
        let s = get_c_string_from_tagged(val);
        !s.is_empty()
    } else if tag == 0xFFF8_0000_0000_0000 {
        true // symbols are always truthy
    } else if tag == 0xFFF6_0000_0000_0000 || tag == 0xFFFB_0000_0000_0000 || tag == 0xFFF9_0000_0000_0000 || tag == 0xFFFA_0000_0000_0000 || tag == 0xFFFC_0000_0000_0000 {
        true
    } else {
        let f = f64::from_bits(val);
        f != 0.0 && !f.is_nan()
    };
    if b { 0xFFF4_0000_0000_0000 } else { 0xFFF3_0000_0000_0000 }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Object(val: u64) -> u64 {
    let tag = val & 0xFFFF_0000_0000_0000;
    if tag == 0xFFF1_0000_0000_0000 || tag == 0xFFF2_0000_0000_0000 {
        __bs_new_object()
    } else if tag == 0xFFF6_0000_0000_0000 || tag == 0xFFFB_0000_0000_0000 || tag == 0xFFF9_0000_0000_0000 || tag == 0xFFFA_0000_0000_0000 || tag == 0xFFFC_0000_0000_0000 {
        val
    } else if tag == 0xFFF7_0000_0000_0000 {
        __bs_String_new_1(val)
    } else if tag == 0xFFF3_0000_0000_0000 || tag == 0xFFF4_0000_0000_0000 {
        __bs_Boolean_new_1(val)
    } else {
        __bs_Number_new_1(val)
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Date(_val: u64) -> u64 {
    let now = std::time::SystemTime::now();
    let since_the_epoch = now.duration_since(std::time::UNIX_EPOCH).expect("Time went backwards");
    let ms = since_the_epoch.as_millis() as f64;
    create_tagged_string(&crate::dynamic_call::date_to_string(ms))
}

// ===========================================================================
// Builtin Constructors (called with new)
// ===========================================================================

#[no_mangle]
pub unsafe extern "C" fn __bs_Object_new(val: u64) -> u64 {
    __bs_Object(val)
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Object_new_0() -> u64 {
    __bs_new_object()
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Object_new_1(val: u64) -> u64 {
    __bs_Object(val)
}

#[no_mangle]
pub unsafe extern "C" fn __bs_String_new(val: u64) -> u64 {
    if (val & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 {
        __bs_String_new_0()
    } else {
        __bs_String_new_1(val)
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_String_new_0() -> u64 {
    let obj = __bs_alloc(&STRING_VTABLE, 8);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    let s_prim = create_tagged_string("");
    set_dynamic_property(payload as *mut u8, "[[PrimitiveValue]]".to_string(), s_prim);
    set_dynamic_property(payload as *mut u8, "length".to_string(), gc::box_number(0.0));
    obj
}

#[no_mangle]
pub unsafe extern "C" fn __bs_String_new_1(val: u64) -> u64 {
    let s_prim = __bs_String(val);
    let obj = __bs_alloc(&STRING_VTABLE, 8);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    set_dynamic_property(payload as *mut u8, "[[PrimitiveValue]]".to_string(), s_prim);
    
    let s_str = get_c_string_from_tagged(s_prim);
    set_dynamic_property(payload as *mut u8, "length".to_string(), gc::box_number(s_str.len() as f64));
    
    let chars: Vec<char> = s_str.chars().collect();
    for (i, ch) in chars.iter().enumerate() {
        let ch_str = ch.to_string();
        let ch_tagged = create_tagged_string(&ch_str);
        set_dynamic_property(payload as *mut u8, i.to_string(), ch_tagged);
    }
    obj
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Number_new(val: u64) -> u64 {
    if (val & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 {
        __bs_Number_new_0()
    } else {
        __bs_Number_new_1(val)
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Number_new_0() -> u64 {
    let obj = __bs_alloc(&NUMBER_VTABLE, 8);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    set_dynamic_property(payload as *mut u8, "[[PrimitiveValue]]".to_string(), gc::box_number(0.0));
    obj
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Number_new_1(val: u64) -> u64 {
    let n_prim = __bs_Number(val);
    let obj = __bs_alloc(&NUMBER_VTABLE, 8);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    set_dynamic_property(payload as *mut u8, "[[PrimitiveValue]]".to_string(), n_prim);
    obj
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Boolean_new(val: u64) -> u64 {
    if (val & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 {
        __bs_Boolean_new_0()
    } else {
        __bs_Boolean_new_1(val)
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Boolean_new_0() -> u64 {
    let obj = __bs_alloc(&BOOLEAN_VTABLE, 8);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    set_dynamic_property(payload as *mut u8, "[[PrimitiveValue]]".to_string(), 0xFFF3_0000_0000_0000); // false
    obj
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Boolean_new_1(val: u64) -> u64 {
    let b_prim = __bs_Boolean(val);
    let obj = __bs_alloc(&BOOLEAN_VTABLE, 8);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    set_dynamic_property(payload as *mut u8, "[[PrimitiveValue]]".to_string(), b_prim);
    obj
}

fn parse_date_string(s: &str) -> f64 {
    if let Ok(ms) = s.parse::<f64>() {
        return ms;
    }
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() >= 3 {
        let t_parts: Vec<&str> = parts[2].split('T').collect();
        let day_str = t_parts.first().copied().unwrap_or("");
        let time_str = t_parts.get(1).copied().unwrap_or("");

        if let (Ok(y), Ok(m), Ok(d)) = (parts[0].parse::<i32>(), parts[1].parse::<u32>(), day_str.parse::<u32>()) {
            let y_from_epoch = y - 1970;
            let leap_years = if y_from_epoch >= 0 {
                (y_from_epoch + 1) / 4
            } else {
                (y_from_epoch - 2) / 4
            };
            let days_in_months = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
            let mut days = y_from_epoch * 365 + leap_years;
            for i in 0..(m.saturating_sub(1) as usize) {
                if i < 12 {
                    days += days_in_months[i];
                }
            }
            if m > 2 && y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
                days += 1;
            }
            days += (d as i32) - 1;

            let mut ms = (days as f64) * 86400.0 * 1000.0;

            if !time_str.is_empty() {
                let hms_str = time_str.split(|c: char| c == 'Z' || c == '+' || c == '-').next().unwrap_or("");
                let hms_parts: Vec<&str> = hms_str.split(':').collect();
                let hours = hms_parts.get(0).copied().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                let minutes = hms_parts.get(1).copied().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                let seconds = hms_parts.get(2).copied().unwrap_or("0").parse::<f64>().unwrap_or(0.0);

                ms += hours * 3600.0 * 1000.0;
                ms += minutes * 60.0 * 1000.0;
                ms += seconds * 1000.0;
            }
            return ms;
        }
    }
    0.0
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Date_new(val: u64) -> u64 {
    if (val & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 {
        __bs_Date_new_0()
    } else {
        __bs_Date_new_1(val)
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Date_new_0() -> u64 {
    let now = std::time::SystemTime::now();
    let since_the_epoch = now.duration_since(std::time::UNIX_EPOCH).expect("Time went backwards");
    let ms = since_the_epoch.as_millis() as f64;
    let obj = __bs_alloc(&DATE_VTABLE, 8);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    set_dynamic_property(payload as *mut u8, "[[PrimitiveValue]]".to_string(), gc::box_number(ms));
    obj
}

#[no_mangle]
pub unsafe extern "C" fn __bs_RegExp_new(pattern_tagged: u64, flags_tagged: u64) -> u64 {
    let obj = __bs_new_object();
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    set_dynamic_property(payload as *mut u8, "source".to_string(), pattern_tagged);
    set_dynamic_property(payload as *mut u8, "flags".to_string(), flags_tagged);
    obj
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Date_new_1(val: u64) -> u64 {
    let ms = if (val & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 {
        f64::NAN
    } else if (val & 0xFFFF_0000_0000_0000) == 0xFFF2_0000_0000_0000 {
        0.0
    } else {
        let tag = val & 0xFFFF_0000_0000_0000;
        if tag == 0xFFF7_0000_0000_0000 {
            let s = get_c_string_from_tagged(val);
            parse_date_string(s)
        } else {
            f64::from_bits(val)
        }
    };
    let obj = __bs_alloc(&DATE_VTABLE, 8);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    set_dynamic_property(payload as *mut u8, "[[PrimitiveValue]]".to_string(), gc::box_number(ms));
    obj
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Date_new_n(
    y_tagged: u64,
    m_tagged: u64,
    d_tagged: u64,
    h_tagged: u64,
    min_tagged: u64,
    s_tagged: u64,
    ms_tagged: u64,
) -> u64 {
    let y = if (y_tagged & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 { 1970 } else { f64::from_bits(y_tagged) as i32 };
    let m = if (m_tagged & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 { 0 } else { f64::from_bits(m_tagged) as u32 };
    let d = if (d_tagged & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 { 1 } else { f64::from_bits(d_tagged) as u32 };
    let h = if (h_tagged & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 { 0 } else { f64::from_bits(h_tagged) as u32 };
    let min = if (min_tagged & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 { 0 } else { f64::from_bits(min_tagged) as u32 };
    let s = if (s_tagged & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 { 0 } else { f64::from_bits(s_tagged) as u32 };
    let ms_val = if (ms_tagged & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 { 0.0 } else { f64::from_bits(ms_tagged) };

    let adjusted_y = if y >= 0 && y <= 99 { 1900 + y } else { y };

    let y_from_epoch = adjusted_y - 1970;
    let leap_years = if y_from_epoch >= 0 {
        (y_from_epoch + 1) / 4
    } else {
        (y_from_epoch - 2) / 4
    };
    let days_in_months = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut days = y_from_epoch * 365 + leap_years;
    for i in 0..(m as usize) {
        if i < 12 {
            days += days_in_months[i];
        }
    }
    let is_leap = adjusted_y % 4 == 0 && (adjusted_y % 100 != 0 || adjusted_y % 400 == 0);
    if m > 1 && is_leap {
        days += 1;
    }
    days += (d as i32) - 1;

    let mut epoch_ms = (days as f64) * 86400.0 * 1000.0;
    epoch_ms += (h as f64) * 3600.0 * 1000.0;
    epoch_ms += (min as f64) * 60.0 * 1000.0;
    epoch_ms += (s as f64) * 1000.0;
    epoch_ms += ms_val;

    let obj = __bs_alloc(&DATE_VTABLE, 8);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    set_dynamic_property(payload as *mut u8, "[[PrimitiveValue]]".to_string(), gc::box_number(epoch_ms));
    obj
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Array_new(val: u64) -> u64 {
    let tag = val & 0xFFFF_0000_0000_0000;
    if tag == 0xFFF1_0000_0000_0000 {
        __bs_array_new()
    } else if tag == 0 || (tag > 0 && tag < 0xFFF0_0000_0000_0000) {
        let f = f64::from_bits(val);
        let len = f as u32;
        let tagged = __bs_array_new();
        let arr = crate::array::untag_array(tagged);
        crate::array::grow_array(arr, len);
        (*arr).length = len;
        for i in 0..len {
            *(*arr).data.add(i as usize) = 0xFFF1_0000_0000_0000; // undefined
        }
        tagged
    } else {
        let tagged = __bs_array_new();
        crate::array::__bs_array_push(tagged, val);
        tagged
    }
}

// ===========================================================================
// Builtin Objects Static Methods
// ===========================================================================

#[no_mangle]
pub unsafe extern "C" fn __bs_date_now() -> u64 {
    let now = std::time::SystemTime::now();
    let since_the_epoch = now.duration_since(std::time::UNIX_EPOCH).expect("Time went backwards");
    gc::box_number(since_the_epoch.as_millis() as f64)
}

#[no_mangle]
pub unsafe extern "C" fn __bs_string_fromCharCode(code: u64) -> u64 {
    let f = f64::from_bits(code);
    let ch = (f as u32).try_into().unwrap_or('\0');
    let ch_str = ch.to_string();
    create_tagged_string(&ch_str)
}

#[no_mangle]
pub unsafe extern "C" fn __bs_string_fromCodePoint(code: u64) -> u64 {
    __bs_string_fromCharCode(code)
}

#[no_mangle]
pub unsafe extern "C" fn __bs_object_keys(obj: u64) -> u64 {
    let tag = obj & 0xFFFF_0000_0000_0000;
    let array = __bs_array_new();
    if tag == 0xFFF6_0000_0000_0000 {
        let payload = obj & 0x0000_FFFF_FFFF_FFFF;
        if payload != 0 {
            let obj_ptr = payload as *mut u8;
            let vtable_ptr = *(obj_ptr as *const *const VTable);
            if !vtable_ptr.is_null() {
                let vtable = &*vtable_ptr;
                let fields_count = vtable.fields_count as usize;
                if fields_count > 0 && !vtable.field_names.is_null() {
                    for i in 0..fields_count {
                        let name_ptr = *vtable.field_names.add(i);
                        if !name_ptr.is_null() {
                            let name_cstr = std::ffi::CStr::from_ptr(name_ptr as *const libc::c_char);
                            let name_str = name_cstr.to_str().unwrap_or("");
                            if !name_str.starts_with("__") && name_str != "[[PrimitiveValue]]" {
                                let k_tagged = create_tagged_string(name_str);
                                crate::array::__bs_array_push(array, k_tagged);
                            }
                        }
                    }
                }
            }
            let map = DYNAMIC_PROPERTIES.lock().unwrap();
            if let Some(obj_entry) = map.get(&(obj_ptr as usize)) {
                for k in obj_entry.keys() {
                    if k != "[[PrimitiveValue]]" && !k.starts_with("__") {
                        let k_tagged = create_tagged_string(k);
                        crate::array::__bs_array_push(array, k_tagged);
                    }
                }
            }
        }
    }
    array
}

#[no_mangle]
pub unsafe extern "C" fn __bs_object_rest(obj: u64, excluded_arr: u64) -> u64 {
    let new_obj = __bs_new_object();
    let new_payload = new_obj & 0x0000_FFFF_FFFF_FFFF;
    
    let mut excluded_keys = Vec::new();
    let arr = array::untag_array(excluded_arr);
    if !arr.is_null() {
        let len = (*arr).length as usize;
        for i in 0..len {
            let key_val = *(*arr).data.add(i);
            if (key_val & 0xFFFF_0000_0000_0000) == 0xFFF7_0000_0000_0000 {
                let key_str = get_c_string_from_tagged(key_val);
                excluded_keys.push(key_str.to_string());
            }
        }
    }

    let mut props_to_copy = Vec::new();

    let tag = obj & 0xFFFF_0000_0000_0000;
    if tag == 0xFFF6_0000_0000_0000 {
        let payload = obj & 0x0000_FFFF_FFFF_FFFF;
        if payload != 0 {
            let obj_ptr = payload as *mut u8;
            
            let vtable_ptr = *(obj_ptr as *const *const VTable);
            if !vtable_ptr.is_null() {
                let vtable = &*vtable_ptr;
                let fields_count = vtable.fields_count as usize;
                if fields_count > 0 && !vtable.field_names.is_null() {
                    for i in 0..fields_count {
                        let name_ptr = *vtable.field_names.add(i);
                        if !name_ptr.is_null() {
                            let name_cstr = std::ffi::CStr::from_ptr(name_ptr as *const libc::c_char);
                            let name_str = name_cstr.to_str().unwrap_or("");
                            if !name_str.starts_with("__") && name_str != "[[PrimitiveValue]]" {
                                if !excluded_keys.contains(&name_str.to_string()) {
                                    let field_slot = (obj_ptr as *const u64).add(1 + i);
                                    let val = *field_slot;
                                    props_to_copy.push((name_str.to_string(), val));
                                }
                            }
                        }
                    }
                }
            }
            
            {
                let map = DYNAMIC_PROPERTIES.lock().unwrap();
                if let Some(obj_entry) = map.get(&(obj_ptr as usize)) {
                    for (k, &val) in obj_entry.iter() {
                        if k != "[[PrimitiveValue]]" && !k.starts_with("__") {
                            if !excluded_keys.contains(k) {
                                props_to_copy.push((k.clone(), val));
                            }
                        }
                    }
                }
            }
        }
    }

    for (k, val) in props_to_copy {
        set_dynamic_property(new_payload as *mut u8, k, val);
    }
    
    new_obj
}

#[no_mangle]
pub unsafe extern "C" fn __bs_object_values(obj: u64) -> u64 {
    let tag = obj & 0xFFFF_0000_0000_0000;
    let array = __bs_array_new();
    if tag == 0xFFF6_0000_0000_0000 {
        let payload = obj & 0x0000_FFFF_FFFF_FFFF;
        if payload != 0 {
            let obj_ptr = payload as *mut u8;
            let vtable_ptr = *(obj_ptr as *const *const VTable);
            if !vtable_ptr.is_null() {
                let vtable = &*vtable_ptr;
                let fields_count = vtable.fields_count as usize;
                if fields_count > 0 && !vtable.field_names.is_null() {
                    for i in 0..fields_count {
                        let name_ptr = *vtable.field_names.add(i);
                        if !name_ptr.is_null() {
                            let name_cstr = std::ffi::CStr::from_ptr(name_ptr as *const libc::c_char);
                            let name_str = name_cstr.to_str().unwrap_or("");
                            if !name_str.starts_with("__") && name_str != "[[PrimitiveValue]]" {
                                let val_ptr = (obj_ptr as *const u64).add(1 + i);
                                crate::array::__bs_array_push(array, *val_ptr);
                            }
                        }
                    }
                }
            }
            let map = DYNAMIC_PROPERTIES.lock().unwrap();
            if let Some(obj_entry) = map.get(&(obj_ptr as usize)) {
                for (k, v) in obj_entry {
                    if k != "[[PrimitiveValue]]" && !k.starts_with("__") {
                        crate::array::__bs_array_push(array, *v);
                    }
                }
            }
        }
    }
    array
}

#[no_mangle]
pub unsafe extern "C" fn __bs_object_entries(obj: u64) -> u64 {
    let tag = obj & 0xFFFF_0000_0000_0000;
    let array = __bs_array_new();
    if tag == 0xFFF6_0000_0000_0000 {
        let payload = obj & 0x0000_FFFF_FFFF_FFFF;
        if payload != 0 {
            let obj_ptr = payload as *mut u8;
            let push_entry = |arr, k: &str, v: u64| {
                let entry = __bs_array_new();
                crate::array::__bs_array_push(entry, create_tagged_string(k));
                crate::array::__bs_array_push(entry, v);
                crate::array::__bs_array_push(arr, entry);
            };
            let vtable_ptr = *(obj_ptr as *const *const VTable);
            if !vtable_ptr.is_null() {
                let vtable = &*vtable_ptr;
                let fields_count = vtable.fields_count as usize;
                if fields_count > 0 && !vtable.field_names.is_null() {
                    for i in 0..fields_count {
                        let name_ptr = *vtable.field_names.add(i);
                        if !name_ptr.is_null() {
                            let name_cstr = std::ffi::CStr::from_ptr(name_ptr as *const libc::c_char);
                            let name_str = name_cstr.to_str().unwrap_or("");
                            if !name_str.starts_with("__") && name_str != "[[PrimitiveValue]]" {
                                let val_ptr = (obj_ptr as *const u64).add(1 + i);
                                push_entry(array, name_str, *val_ptr);
                            }
                        }
                    }
                }
            }
            let map = DYNAMIC_PROPERTIES.lock().unwrap();
            if let Some(obj_entry) = map.get(&(obj_ptr as usize)) {
                for (k, v) in obj_entry {
                    if k != "[[PrimitiveValue]]" && !k.starts_with("__") {
                        push_entry(array, k, *v);
                    }
                }
            }
        }
    }
    array
}

#[no_mangle]
pub unsafe extern "C" fn __bs_object_assign(target: u64, source: u64) -> u64 {
    let target_tag = target & 0xFFFF_0000_0000_0000;
    let source_tag = source & 0xFFFF_0000_0000_0000;
    if target_tag == 0xFFF6_0000_0000_0000 {
        let target_payload = target & 0x0000_FFFF_FFFF_FFFF;
        let target_ptr = target_payload as *mut u8;
        if source_tag == 0xFFF6_0000_0000_0000 {
            let source_payload = source & 0x0000_FFFF_FFFF_FFFF;
            let source_ptr = source_payload as *mut u8;
            let vtable_ptr = *(source_ptr as *const *const VTable);
            if !vtable_ptr.is_null() {
                let vtable = &*vtable_ptr;
                let fields_count = vtable.fields_count as usize;
                if fields_count > 0 && !vtable.field_names.is_null() {
                    for i in 0..fields_count {
                        let name_ptr = *vtable.field_names.add(i);
                        if !name_ptr.is_null() {
                            let name_cstr = std::ffi::CStr::from_ptr(name_ptr as *const libc::c_char);
                            let name_str = name_cstr.to_str().unwrap_or("");
                            if !name_str.starts_with("__") && name_str != "[[PrimitiveValue]]" {
                                let val_ptr = (source_ptr as *const u64).add(1 + i);
                                crate::json_tape::__bs_prop_set(target, name_ptr, name_str.len() as u32, *val_ptr);
                            }
                        }
                    }
                }
            }
            let map = DYNAMIC_PROPERTIES.lock().unwrap();
            if let Some(obj_entry) = map.get(&(source_ptr as usize)) {
                let copy_list: Vec<(String, u64)> = obj_entry.iter().map(|(k, v)| (k.clone(), *v)).collect();
                drop(map);
                for (k, v) in copy_list {
                    if k != "[[PrimitiveValue]]" && !k.starts_with("__") {
                        set_dynamic_property(target_ptr, k, v);
                    }
                }
            }
        }
    }
    target
}

#[no_mangle]
pub unsafe extern "C" fn __bs_object_create(proto: u64) -> u64 {
    let obj = __bs_new_object();
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    set_dynamic_property(payload as *mut u8, "__proto__".to_string(), proto);
    obj
}

#[no_mangle]
pub unsafe extern "C" fn __bs_object_getPrototypeOf(obj: u64) -> u64 {
    let tag = obj & 0xFFFF_0000_0000_0000;
    if tag == 0xFFF6_0000_0000_0000 {
        let payload = obj & 0x0000_FFFF_FFFF_FFFF;
        let obj_ptr = payload as *mut u8;
        if let Some(proto) = get_dynamic_property(obj_ptr, "__proto__") {
            return proto;
        }
    }
    0xFFF5_0000_0000_0000 // null
}

#[no_mangle]
pub unsafe extern "C" fn __bs_object_fromEntries(pairs: u64) -> u64 {
    let obj = __bs_alloc(&OBJECT_VTABLE, 8);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    
    let tag = pairs & 0xFFFF_0000_0000_0000;
    if tag == 0xFFFB_0000_0000_0000 {
        let arr = crate::array::untag_array(pairs);
        if !arr.is_null() {
            let len = (*arr).length;
            for i in 0..len {
                let entry_tagged = *(*arr).data.add(i as usize);
                let entry_tag = entry_tagged & 0xFFFF_0000_0000_0000;
                if entry_tag == 0xFFFB_0000_0000_0000 {
                    let entry_arr = crate::array::untag_array(entry_tagged);
                    if !entry_arr.is_null() && (*entry_arr).length >= 2 {
                        let key_tagged = *(*entry_arr).data.add(0);
                        let val_tagged = *(*entry_arr).data.add(1);
                        
                        let key_string_tagged = __bs_String(key_tagged);
                        let key_str = get_c_string_from_tagged(key_string_tagged).to_string();
                        
                        set_dynamic_property(payload as *mut u8, key_str, val_tagged);
                    }
                }
            }
        }
    }
    obj
}

static mut GLOBAL_THIS_OBJ: u64 = 0;

#[no_mangle]
pub unsafe extern "C" fn __bs_get_globalThis() -> u64 {
    if GLOBAL_THIS_OBJ == 0 {
        GLOBAL_THIS_OBJ = __bs_alloc(&OBJECT_VTABLE, 8);
    }
    GLOBAL_THIS_OBJ
}

fn encode_uri_str(s: &str, is_component: bool) -> String {
    let mut res = String::new();
    for b in s.bytes() {
        let is_unescaped = match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'!' | b'~' | b'*' | b'\'' | b'(' | b')' => true,
            b';' | b',' | b'/' | b'?' | b':' | b'@' | b'&' | b'=' | b'+' | b'$' | b'#' if !is_component => true,
            _ => false,
        };
        if is_unescaped {
            res.push(b as char);
        } else {
            res.push_str(&format!("%{:02X}", b));
        }
    }
    res
}

fn decode_uri_str(s: &str) -> Option<String> {
    let mut bytes = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '%' {
            if i + 2 < chars.len() {
                let hex = format!("{}{}", chars[i+1], chars[i+2]);
                if let Ok(b) = u8::from_str_radix(&hex, 16) {
                    bytes.push(b);
                    i += 3;
                    continue;
                }
            }
            return None;
        } else {
            let mut buf = [0; 4];
            for &b in chars[i].encode_utf8(&mut buf).as_bytes() {
                bytes.push(b);
            }
            i += 1;
        }
    }
    String::from_utf8(bytes).ok()
}

#[no_mangle]
pub unsafe extern "C" fn __bs_encodeURI(val: u64) -> u64 {
    let s_tagged = __bs_String(val);
    let s = get_c_string_from_tagged(s_tagged);
    let encoded = encode_uri_str(s, false);
    create_tagged_string(&encoded)
}

#[no_mangle]
pub unsafe extern "C" fn __bs_decodeURI(val: u64) -> u64 {
    let s_tagged = __bs_String(val);
    let s = get_c_string_from_tagged(s_tagged);
    if let Some(decoded) = decode_uri_str(s) {
        create_tagged_string(&decoded)
    } else {
        let msg = create_tagged_string("URI malformed");
        crate::exception::__bs_throw(crate::exception::__bs_URIError_new(msg))
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_encodeURIComponent(val: u64) -> u64 {
    let s_tagged = __bs_String(val);
    let s = get_c_string_from_tagged(s_tagged);
    let encoded = encode_uri_str(s, true);
    create_tagged_string(&encoded)
}

#[no_mangle]
pub unsafe extern "C" fn __bs_decodeURIComponent(val: u64) -> u64 {
    let s_tagged = __bs_String(val);
    let s = get_c_string_from_tagged(s_tagged);
    if let Some(decoded) = decode_uri_str(s) {
        create_tagged_string(&decoded)
    } else {
        let msg = create_tagged_string("URI malformed");
        crate::exception::__bs_throw(crate::exception::__bs_URIError_new(msg))
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_strict_eq(l: u64, r: u64) -> u64 {
    const TAG_STRING: u64 = 0xFFF7_0000_0000_0000;
    const TAG_MIN: u64 = 0xFFF1;
    const TAG_TRUE: u64 = 0xFFF4_0000_0000_0000;
    const TAG_FALSE: u64 = 0xFFF3_0000_0000_0000;

    let l_tag = l & 0xFFFF_0000_0000_0000;
    let r_tag = r & 0xFFFF_0000_0000_0000;

    // Check if either is a plain f64 number.
    // Top 16 bits of a number are < TAG_MIN (0xFFF1).
    let l_is_num = (l >> 48) < TAG_MIN;
    let r_is_num = (r >> 48) < TAG_MIN;

    if l_is_num && r_is_num {
        let lf = f64::from_bits(l);
        let rf = f64::from_bits(r);
        // Special check: NaN === NaN is false in JS
        if lf.is_nan() || rf.is_nan() {
            TAG_FALSE
        } else if lf == rf {
            TAG_TRUE
        } else {
            TAG_FALSE
        }
    } else if !l_is_num && !r_is_num {
        if l_tag == TAG_STRING && r_tag == TAG_STRING {
            let ls = get_c_string_from_tagged(l);
            let rs = get_c_string_from_tagged(r);
            if ls == rs {
                TAG_TRUE
            } else {
                TAG_FALSE
            }
        } else {
            if l == r {
                TAG_TRUE
            } else {
                TAG_FALSE
            }
        }
    } else {
        TAG_FALSE
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_strict_ne(l: u64, r: u64) -> u64 {
    const TAG_TRUE: u64 = 0xFFF4_0000_0000_0000;
    const TAG_FALSE: u64 = 0xFFF3_0000_0000_0000;
    if __bs_strict_eq(l, r) == TAG_TRUE {
        TAG_FALSE
    } else {
        TAG_TRUE
    }
}

/// JS `+` operator: numeric addition when both sides are numbers,
/// string concatenation when either side is a string (or coerces to one).
#[no_mangle]
pub unsafe extern "C" fn __bs_add(l: u64, r: u64) -> u64 {
    const TAG_STRING: u64 = 0xFFF7_0000_0000_0000;
    const TAG_MIN: u64    = 0xFFF1;

    let l_is_num = (l >> 48) < TAG_MIN;
    let r_is_num = (r >> 48) < TAG_MIN;

    let l_tag = l & 0xFFFF_0000_0000_0000;
    let r_tag = r & 0xFFFF_0000_0000_0000;

    let l_is_str = l_tag == TAG_STRING;
    let r_is_str = r_tag == TAG_STRING;

    if l_is_num && r_is_num {
        // Both plain numbers — float addition
        let lf = f64::from_bits(l);
        let rf = f64::from_bits(r);
        (lf + rf).to_bits()
    } else if l_is_str || r_is_str {
        // At least one string — coerce both and concatenate
        let ls = get_c_string_from_tagged(__bs_String(l));
        let rs = get_c_string_from_tagged(__bs_String(r));
        let concat = format!("{}{}", ls, rs);
        create_tagged_string(&concat)
    } else {
        // Both are non-string tagged values (null, bool, undefined, object) —
        // coerce to number via f64 bitcast as a best-effort fallback
        let lf = f64::from_bits(l);
        let rf = f64::from_bits(r);
        (lf + rf).to_bits()
    }
}

// ===========================================================================
// Operator Implementations (Stage 15)
// ===========================================================================

#[no_mangle]
pub unsafe extern "C" fn __bs_is_nullish(val: u64) -> u64 {
    const TAG_TRUE: u64 = 0xFFF4_0000_0000_0000;
    const TAG_FALSE: u64 = 0xFFF3_0000_0000_0000;
    let tag = val & 0xFFFF_0000_0000_0000;
    if tag == 0xFFF1_0000_0000_0000 || tag == 0xFFF2_0000_0000_0000 {
        TAG_TRUE
    } else {
        TAG_FALSE
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_exp(l: u64, r: u64) -> u64 {
    let lf = f64::from_bits(__bs_Number(l));
    let rf = f64::from_bits(__bs_Number(r));
    lf.powf(rf).to_bits()
}

// Duplicate removed

#[no_mangle]
pub unsafe extern "C" fn __bs_in(key: u64, obj: u64) -> u64 {
    const TAG_TRUE: u64 = 0xFFF4_0000_0000_0000;
    const TAG_FALSE: u64 = 0xFFF3_0000_0000_0000;
    
    let key_str_tagged = __bs_String(key);
    let key_str = get_c_string_from_tagged(key_str_tagged);

    let tag = obj & 0xFFFF_0000_0000_0000;
    if tag == 0xFFF6_0000_0000_0000 {
        let payload = obj & 0x0000_FFFF_FFFF_FFFF;
        let obj_ptr = payload as *mut u8;
        
        if get_dynamic_property(obj_ptr, key_str).is_some() {
            return TAG_TRUE;
        }

        let vtable_ptr = *(obj_ptr as *const *const VTable);
        if !vtable_ptr.is_null() {
            // Check VTable field names
            let mut i = 0;
            while i < (*vtable_ptr).fields_count {
                let name_cstr = std::ffi::CStr::from_ptr(*(*vtable_ptr).field_names.add(i as usize) as *const libc::c_char);
                if name_cstr.to_str().unwrap_or("") == key_str {
                    return TAG_TRUE;
                }
                i += 1;
            }
        }
    }
    TAG_FALSE
}

#[no_mangle]
pub unsafe extern "C" fn __bs_delete_prop(obj: u64, key: u64) -> u64 {
    const TAG_TRUE: u64 = 0xFFF4_0000_0000_0000;
    
    let key_str_tagged = __bs_String(key);
    let key_str = get_c_string_from_tagged(key_str_tagged);

    let tag = obj & 0xFFFF_0000_0000_0000;
    if tag == 0xFFF6_0000_0000_0000 {
        let payload = obj & 0x0000_FFFF_FFFF_FFFF;
        let obj_ptr = payload as *mut u8;
        delete_dynamic_property(obj_ptr, key_str);
    }
    TAG_TRUE
}

// ===========================================================================
// Symbol support
// ===========================================================================

use std::sync::atomic::{AtomicU64, Ordering};

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

/// Dynamic import runtime stub: returns a resolved Promise containing a fresh empty namespace object.
#[no_mangle]
pub unsafe extern "C" fn __bs_dynamic_import(_specifier: u64) -> u64 {
    let p = crate::promise::__bs_promise_new();
    let obj = __bs_new_object();
    crate::promise::__bs_promise_resolve(p, obj);
    p
}

