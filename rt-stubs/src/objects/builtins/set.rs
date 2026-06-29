use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;



unsafe fn update_size(obj_tagged: u64, size: usize) {
    let payload = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    crate::objects::dynamic_props::set_dynamic_property(payload as *mut u8, "size".to_string(), crate::circ::box_number(size as f64));
}

pub static SET_DATA: Lazy<Mutex<HashMap<u64, HashMap<u64, ()>>>> = Lazy::new(|| Mutex::new(HashMap::new()));


// --- Set Methods ---

pub unsafe extern "C-unwind" fn set_add(env: u64, key: u64) -> u64 {
    let closure_ptr = (env & 0x0000_FFFF_FFFF_FFFF) as *const u64;
    let obj_tagged = *(closure_ptr.add(3));
    let mut data = SET_DATA.lock().unwrap();
    let set = data.entry(obj_tagged).or_insert_with(HashMap::new);
    
    if let Some((old_k, _)) = set.remove_entry(&key) {
        crate::circ::circ_dec_tagged(old_k);
    }
    crate::circ::circ_inc_tagged(key);
    set.insert(key, ());
    
    let size = set.len();
    drop(data);
    update_size(obj_tagged, size);
    crate::circ::circ_inc_tagged(obj_tagged);
    
    obj_tagged
}

pub unsafe extern "C-unwind" fn set_has(env: u64, key: u64) -> u64 {
    let closure_ptr = (env & 0x0000_FFFF_FFFF_FFFF) as *const u64;
    let obj_tagged = *(closure_ptr.add(3));
    let data = SET_DATA.lock().unwrap();
    let has = data.get(&obj_tagged).map_or(false, |set| set.contains_key(&key));
    
    crate::circ::box_boolean(has)
}

pub unsafe extern "C-unwind" fn set_delete(env: u64, key: u64) -> u64 {
    let closure_ptr = (env & 0x0000_FFFF_FFFF_FFFF) as *const u64;
    let obj_tagged = *(closure_ptr.add(3));
    let mut data = SET_DATA.lock().unwrap();
    let mut deleted = false;
    let mut size = 0;
    if let Some(set) = data.get_mut(&obj_tagged) {
        if let Some((old_k, _)) = set.remove_entry(&key) {
            crate::circ::circ_dec_tagged(old_k);
            deleted = true;
        }
        size = set.len();
    }
    drop(data);
    update_size(obj_tagged, size);
    crate::circ::box_boolean(deleted)
}

pub unsafe extern "C-unwind" fn set_clear(env: u64) -> u64 {
    let closure_ptr = (env & 0x0000_FFFF_FFFF_FFFF) as *const u64;
    let obj_tagged = *(closure_ptr.add(3));
    let mut data = SET_DATA.lock().unwrap();
    if let Some(set) = data.get_mut(&obj_tagged) {
        for (k, _) in set.drain() {
            crate::circ::circ_dec_tagged(k);
        }
    }
    drop(data);
    update_size(obj_tagged, 0);
    
    crate::circ::circ_dec_tagged(env);
    
    0xFFF1_0000_0000_0000
}

#[no_mangle]
pub unsafe extern "C-unwind" fn set_drop(obj_ptr: *mut u8) {
    let obj_tagged = obj_ptr as u64 | 0xFFF6_0000_0000_0000;
    // println!("set_drop called for {:x}", obj_tagged);
    let mut data = SET_DATA.lock().unwrap();
    if let Some(set) = data.remove(&obj_tagged) {
        // println!("set_drop: set FOUND with {} elements", set.len());
        for (k, _) in set {
            crate::circ::circ_dec_tagged(k);
        }
    } else {
        // println!("set_drop: set NOT FOUND in SET_DATA!");
    }
}


#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_Set_new_0() -> u64 {
    let obj = crate::__bs_alloc(&crate::SET_VTABLE, 16);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    crate::objects::dynamic_props::set_dynamic_property_moved(payload as *mut u8, "add".to_string(), crate::circ::create_builtin_method(obj, set_add as *const u8));
    crate::objects::dynamic_props::set_dynamic_property_moved(payload as *mut u8, "has".to_string(), crate::circ::create_builtin_method(obj, set_has as *const u8));
    crate::objects::dynamic_props::set_dynamic_property_moved(payload as *mut u8, "delete".to_string(), crate::circ::create_builtin_method(obj, set_delete as *const u8));
    crate::objects::dynamic_props::set_dynamic_property_moved(payload as *mut u8, "clear".to_string(), crate::circ::create_builtin_method(obj, set_clear as *const u8));
    update_size(obj, 0);
    obj
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_Set_new_1(iterable: u64) -> u64 {
    let obj = __bs_Set_new_0();
    if (iterable & 0xFFFF_0000_0000_0000) != 0xFFF1_0000_0000_0000 {
        let payload = obj & 0x0000_FFFF_FFFF_FFFF;
        let add_method = crate::objects::dynamic_props::get_dynamic_property(payload as *mut u8, "add").unwrap_or(0);
        
        let len_f = crate::array::__bs_array_length(iterable);
        let len = f64::from_bits(len_f) as usize;
        for i in 0..len {
            let item = crate::array::__bs_array_get(iterable, crate::circ::box_number(i as f64));
            if add_method != 0 {
                let ret = set_add(add_method, item);
                crate::circ::circ_dec_tagged(ret);
            }
            crate::circ::circ_dec_tagged(item);
        }
    }
    obj
}
