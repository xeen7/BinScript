use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::gc;

unsafe fn create_method(obj_tagged: u64, func_ptr: *const u8) -> u64 {
    let closure_tagged = crate::__bs_alloc_closure(16);
    let closure_ptr = (closure_tagged & 0x0000_FFFF_FFFF_FFFF) as *mut u64;
    *closure_ptr = func_ptr as u64; // offset 0
    *(closure_ptr.add(1)) = obj_tagged; // offset 8
    closure_tagged
}

unsafe fn update_size(obj_tagged: u64, size: usize) {
    let payload = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    crate::objects::dynamic_props::set_dynamic_property(payload as *mut u8, "size".to_string(), gc::box_number(size as f64));
}

pub static SET_DATA: Lazy<Mutex<HashMap<u64, HashMap<u64, ()>>>> = Lazy::new(|| Mutex::new(HashMap::new()));


// --- Set Methods ---

pub unsafe extern "C" fn set_add(env: u64, key: u64) -> u64 {
    let closure_ptr = (env & 0x0000_FFFF_FFFF_FFFF) as *const u64;
    let obj_tagged = *(closure_ptr.add(1));
    let mut data = SET_DATA.lock().unwrap();
    let set = data.entry(obj_tagged).or_insert_with(HashMap::new);
    set.insert(key, ());
    let size = set.len();
    drop(data);
    update_size(obj_tagged, size);
    obj_tagged
}

pub unsafe extern "C" fn set_has(env: u64, key: u64) -> u64 {
    let closure_ptr = (env & 0x0000_FFFF_FFFF_FFFF) as *const u64;
    let obj_tagged = *(closure_ptr.add(1));
    let data = SET_DATA.lock().unwrap();
    let has = data.get(&obj_tagged).map_or(false, |set| set.contains_key(&key));
    gc::box_boolean(has)
}

pub unsafe extern "C" fn set_delete(env: u64, key: u64) -> u64 {
    let closure_ptr = (env & 0x0000_FFFF_FFFF_FFFF) as *const u64;
    let obj_tagged = *(closure_ptr.add(1));
    let mut data = SET_DATA.lock().unwrap();
    let mut deleted = false;
    let mut size = 0;
    if let Some(set) = data.get_mut(&obj_tagged) {
        deleted = set.remove(&key).is_some();
        size = set.len();
    }
    drop(data);
    update_size(obj_tagged, size);
    gc::box_boolean(deleted)
}

pub unsafe extern "C" fn set_clear(env: u64) -> u64 {
    let closure_ptr = (env & 0x0000_FFFF_FFFF_FFFF) as *const u64;
    let obj_tagged = *(closure_ptr.add(1));
    let mut data = SET_DATA.lock().unwrap();
    if let Some(set) = data.get_mut(&obj_tagged) {
        set.clear();
    }
    drop(data);
    update_size(obj_tagged, 0);
    0xFFF1_0000_0000_0000
}


#[no_mangle]
pub unsafe extern "C" fn __bs_Set_new_0() -> u64 {
    let obj = crate::__bs_alloc(&crate::SET_VTABLE, 8);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    crate::objects::dynamic_props::set_dynamic_property(payload as *mut u8, "add".to_string(), create_method(obj, set_add as *const u8));
    crate::objects::dynamic_props::set_dynamic_property(payload as *mut u8, "has".to_string(), create_method(obj, set_has as *const u8));
    crate::objects::dynamic_props::set_dynamic_property(payload as *mut u8, "delete".to_string(), create_method(obj, set_delete as *const u8));
    crate::objects::dynamic_props::set_dynamic_property(payload as *mut u8, "clear".to_string(), create_method(obj, set_clear as *const u8));
    update_size(obj, 0);
    obj
}

#[no_mangle]
pub unsafe extern "C" fn __bs_Set_new_1(iterable: u64) -> u64 {
    let obj = __bs_Set_new_0();
    // Populate from iterable (array of values)
    if (iterable & 0xFFFF_0000_0000_0000) == 0xFFFB_0000_0000_0000 {
        let len_f = crate::array::__bs_array_length(iterable);
        let len = f64::from_bits(len_f) as usize;
        for i in 0..len {
            let item = crate::array::__bs_array_get(iterable, gc::box_number(i as f64));
            set_add(create_method(obj, set_add as *const u8), item);
        }
    }
    obj
}
