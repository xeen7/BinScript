use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub static DYNAMIC_PROPERTIES: Lazy<Mutex<HashMap<usize, HashMap<String, u64>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub unsafe fn set_dynamic_property(obj_ptr: *mut u8, prop_name: String, val_tagged: u64) {
    crate::circ::circ_inc_tagged(val_tagged);
    let old_val = {
        let mut map = DYNAMIC_PROPERTIES.lock().unwrap();
        let obj_entry = map.entry(obj_ptr as usize).or_insert_with(HashMap::new);
        obj_entry.insert(prop_name, val_tagged)
    };
    if let Some(old_val) = old_val {
        crate::circ::circ_dec_tagged(old_val);
    }
}

pub unsafe fn get_dynamic_property(obj_ptr: *mut u8, prop_name: &str) -> Option<u64> {
    let map = DYNAMIC_PROPERTIES.lock().unwrap();
    if let Some(obj_entry) = map.get(&(obj_ptr as usize)) {
        if let Some(&val) = obj_entry.get(prop_name) {
            return Some(val);
        }
    }
    None
}

pub unsafe fn delete_dynamic_property(obj_ptr: *mut u8, prop_name: &str) -> bool {
    let removed = {
        let mut map = DYNAMIC_PROPERTIES.lock().unwrap();
        if let Some(obj_entry) = map.get_mut(&(obj_ptr as usize)) {
            obj_entry.remove(prop_name)
        } else {
            None
        }
    };
    if let Some(old_val) = removed {
        crate::circ::circ_dec_tagged(old_val);
        return true;
    }
    false
}

pub unsafe fn remove_dynamic_properties(obj_ptr: *mut u8) {
    let removed = {
        let mut map = DYNAMIC_PROPERTIES.lock().unwrap();
        map.remove(&(obj_ptr as usize))
    };
    if let Some(entry) = removed {
        for val in entry.values() {
            crate::circ::circ_dec_tagged(*val);
        }
    }
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
        // gc::gc_mark_value(val);
    }
}

pub unsafe fn set_inline_property(props_slot: *mut *mut std::collections::HashMap<String, u64>, prop_name: String, val_tagged: u64) {
    if (*props_slot).is_null() {
        let bx = Box::new(std::collections::HashMap::new());
        *props_slot = Box::into_raw(bx);
    }
    crate::circ::circ_inc_tagged(val_tagged);
    if let Some(old_val) = (**props_slot).insert(prop_name, val_tagged) {
        crate::circ::circ_dec_tagged(old_val);
    }
}

pub unsafe fn get_inline_property(props_slot: *mut *mut std::collections::HashMap<String, u64>, prop_name: &str) -> Option<u64> {
    if (*props_slot).is_null() {
        None
    } else {
        if let Some(&val) = (**props_slot).get(prop_name) {
            // println!("get_inline_property returning val: {} for prop {}", val, prop_name);
            Some(val)
        } else {
            None
        }
    }
}

pub unsafe fn free_inline_properties(props_slot: *mut *mut std::collections::HashMap<String, u64>) {
    if !(*props_slot).is_null() {
        let bx = Box::from_raw(*props_slot);
        // println!("free_inline_properties: freeing {} properties", bx.len());
        for val in bx.values() {
            // println!("  freeing value {}", val);
            crate::circ::circ_dec_tagged(*val);
        }
        *props_slot = std::ptr::null_mut();
    } else {
        // println!("free_inline_properties: slot is null");
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn set_dynamic_property_moved(obj_ptr: *mut u8, prop_name: String, val_tagged: u64) {
    // Move semantics: DO NOT increment reference count of val_tagged
    let old_val = {
        let mut map = DYNAMIC_PROPERTIES.lock().unwrap();
        let obj_entry = map.entry(obj_ptr as usize).or_insert_with(HashMap::new);
        obj_entry.insert(prop_name, val_tagged)
    };
    if let Some(old_val) = old_val {
        crate::circ::circ_dec_tagged(old_val);
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn set_inline_property_moved(props_slot: *mut *mut std::collections::HashMap<String, u64>, prop_name: String, val_tagged: u64) {
    if (*props_slot).is_null() {
        let bx = Box::new(std::collections::HashMap::new());
        *props_slot = Box::into_raw(bx);
    }
    // Move semantics: DO NOT increment reference count of val_tagged
    if let Some(old_val) = (**props_slot).insert(prop_name, val_tagged) {
        crate::circ::circ_dec_tagged(old_val);
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_cleanup_dynamic_properties() {
    let owned_map = {
        let mut map = DYNAMIC_PROPERTIES.lock().unwrap();
        std::mem::take(&mut *map)
    };
    // println!("__bs_cleanup_dynamic_properties: cleaning up {} objects", owned_map.len());
    for entry in owned_map.values() {
        // println!("__bs_cleanup_dynamic_properties: cleaning up {} properties", entry.len());
        for val in entry.values() {
            crate::circ::circ_dec_tagged(*val);
        }
    }
}
