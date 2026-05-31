use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use crate::gc;

pub static DYNAMIC_PROPERTIES: Lazy<Mutex<HashMap<usize, HashMap<String, u64>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

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
