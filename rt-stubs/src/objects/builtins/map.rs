use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;



unsafe fn update_size(obj_tagged: u64, size: usize) {
    let payload = obj_tagged & 0x0000_FFFF_FFFF_FFFF;
    crate::objects::dynamic_props::set_dynamic_property(payload as *mut u8, "size".to_string(), crate::circ::box_number(size as f64));
}

pub static MAP_DATA: Lazy<Mutex<HashMap<u64, HashMap<u64, u64>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// --- Map Methods ---

pub unsafe extern "C-unwind" fn map_set(env: u64, key: u64, val: u64) -> u64 {
    let closure_ptr = (env & 0x0000_FFFF_FFFF_FFFF) as *const u64;
    let obj_tagged = *(closure_ptr.add(3));
    let fmt = b"map_set: map=%lx key=%lx val=%lx\n\0".as_ptr() as *const libc::c_char;
    libc::printf(fmt, obj_tagged, key, val);
    libc::fflush(std::ptr::null_mut());

    let mut data = MAP_DATA.lock().unwrap();
    let map = data.entry(obj_tagged).or_insert_with(HashMap::new);
    
    if let Some((old_k, old_v)) = map.remove_entry(&key) {
        crate::circ::circ_dec_tagged(old_k);
        crate::circ::circ_dec_tagged(old_v);
    }
    crate::circ::circ_inc_tagged(key);
    crate::circ::circ_inc_tagged(val);
    map.insert(key, val);
    
    let size = map.len();
    drop(data);
    update_size(obj_tagged, size);
    crate::circ::circ_inc_tagged(obj_tagged);
    
    obj_tagged
}

pub unsafe extern "C-unwind" fn map_get(env: u64, key: u64) -> u64 {
    let closure_ptr = (env & 0x0000_FFFF_FFFF_FFFF) as *const u64;
    let obj_tagged = *(closure_ptr.add(3));
    let fmt = b"map_get: map=%lx key=%lx\n\0".as_ptr() as *const libc::c_char;
    libc::printf(fmt, obj_tagged, key);
    libc::fflush(std::ptr::null_mut());

    let data = MAP_DATA.lock().unwrap();
    let result = if let Some(map) = data.get(&obj_tagged) {
        if let Some(&val) = map.get(&key) {
            crate::circ::circ_inc_tagged(val);
            val
        } else {
            0xFFF1_0000_0000_0000
        }
    } else {
        0xFFF1_0000_0000_0000 // undefined
    };
    
    result
}

pub unsafe extern "C-unwind" fn map_has(env: u64, key: u64) -> u64 {
    let closure_ptr = (env & 0x0000_FFFF_FFFF_FFFF) as *const u64;
    let obj_tagged = *(closure_ptr.add(3));
    let data = MAP_DATA.lock().unwrap();
    let has = data.get(&obj_tagged).map_or(false, |map| map.contains_key(&key));
    
    crate::circ::box_boolean(has)
}

pub unsafe extern "C-unwind" fn map_delete(env: u64, key: u64) -> u64 {
    let closure_ptr = (env & 0x0000_FFFF_FFFF_FFFF) as *const u64;
    let obj_tagged = *(closure_ptr.add(3));
    let mut data = MAP_DATA.lock().unwrap();
    let mut deleted = false;
    let mut size = 0;
    if let Some(map) = data.get_mut(&obj_tagged) {
        if let Some((old_k, old_v)) = map.remove_entry(&key) {
            crate::circ::circ_dec_tagged(old_k);
            crate::circ::circ_dec_tagged(old_v);
            deleted = true;
        }
        size = map.len();
    }
    drop(data);
    update_size(obj_tagged, size);
    
    crate::circ::box_boolean(deleted)
}

pub unsafe extern "C-unwind" fn map_clear(env: u64) -> u64 {
    let closure_ptr = (env & 0x0000_FFFF_FFFF_FFFF) as *const u64;
    let obj_tagged = *(closure_ptr.add(3));
    let mut data = MAP_DATA.lock().unwrap();
    if let Some(map) = data.get_mut(&obj_tagged) {
        for (k, v) in map.drain() {
            crate::circ::circ_dec_tagged(k);
            crate::circ::circ_dec_tagged(v);
        }
    }
    drop(data);
    update_size(obj_tagged, 0);
    
    0xFFF1_0000_0000_0000 // undefined
}

#[no_mangle]
pub unsafe extern "C-unwind" fn map_drop(obj_ptr: *mut u8) {
    let obj_tagged = obj_ptr as u64 | 0xFFF6_0000_0000_0000;
    // println!("map_drop called for {:x}", obj_tagged);
    let mut data = MAP_DATA.lock().unwrap();
    if let Some(map) = data.remove(&obj_tagged) {
        // println!("map_drop: map FOUND with {} elements", map.len());
        for (k, v) in map {
            crate::circ::circ_dec_tagged(k);
            crate::circ::circ_dec_tagged(v);
        }
    } else {
        // println!("map_drop: map NOT FOUND in MAP_DATA!");
    }
}


// --- Constructors ---

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_Map_new_0() -> u64 {
    let obj = crate::__bs_alloc(&crate::MAP_VTABLE, 16);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    crate::objects::dynamic_props::set_dynamic_property_moved(payload as *mut u8, "set".to_string(), crate::circ::create_builtin_method(obj, map_set as *const u8));
    crate::objects::dynamic_props::set_dynamic_property_moved(payload as *mut u8, "get".to_string(), crate::circ::create_builtin_method(obj, map_get as *const u8));
    crate::objects::dynamic_props::set_dynamic_property_moved(payload as *mut u8, "has".to_string(), crate::circ::create_builtin_method(obj, map_has as *const u8));
    crate::objects::dynamic_props::set_dynamic_property_moved(payload as *mut u8, "delete".to_string(), crate::circ::create_builtin_method(obj, map_delete as *const u8));
    crate::objects::dynamic_props::set_dynamic_property_moved(payload as *mut u8, "clear".to_string(), crate::circ::create_builtin_method(obj, map_clear as *const u8));
    update_size(obj, 0);
    obj
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_Map_new_1(iterable: u64) -> u64 {
    let obj = __bs_Map_new_0();
    if (iterable & 0xFFFF_0000_0000_0000) != 0xFFF1_0000_0000_0000 {
        let payload = obj & 0x0000_FFFF_FFFF_FFFF;
        let set_method = crate::objects::dynamic_props::get_dynamic_property(payload as *mut u8, "set").unwrap_or(0);
        
        let len_f = crate::array::__bs_array_length(iterable);
        let len = f64::from_bits(len_f) as usize;
        for i in 0..len {
            let pair = crate::array::__bs_array_get(iterable, crate::circ::box_number(i as f64));
            if set_method != 0 {
                let k = crate::array::__bs_array_get(pair, crate::circ::box_number(0.0));
                let v = crate::array::__bs_array_get(pair, crate::circ::box_number(1.0));
                let ret = map_set(set_method, k, v);
                crate::circ::circ_dec_tagged(ret);
                crate::circ::circ_dec_tagged(k);
                crate::circ::circ_dec_tagged(v);
            }
            crate::circ::circ_dec_tagged(pair);
        }
    }
    obj
}
