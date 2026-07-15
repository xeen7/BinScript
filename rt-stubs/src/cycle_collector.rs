use std::cell::RefCell;
use crate::circ::{CircHeader, COLOR_BLACK, COLOR_GRAY, COLOR_WHITE, COLOR_PURPLE};

#[derive(Clone, Copy)]
pub struct CircPtr(pub *mut CircHeader);

thread_local! {
    static LOCAL_GARBAGE: RefCell<Vec<CircPtr>> = RefCell::new(Vec::new());
}

#[no_mangle]
pub extern "C-unwind" fn __bs_cycle_collector_init() {
    // Thread-local garbage collection doesn't need a global background thread
}

pub fn push_to_local_queue(batch: &mut Vec<*mut CircHeader>) {
    LOCAL_GARBAGE.with(|garbage| {
        let mut g = garbage.borrow_mut();
        g.extend(batch.iter().map(|&p| CircPtr(p)));
    });
}

#[no_mangle]
pub extern "C-unwind" fn __bs_cycle_collector_flush() {
    // eprintln!("__bs_cycle_collector_flush called");
    let mut work_list = Vec::new();
    LOCAL_GARBAGE.with(|garbage| {
        std::mem::swap(&mut work_list, &mut garbage.borrow_mut());
    });
    
    if !work_list.is_empty() {
        collect_cycles(work_list);
    }
}

// ── Bacon-Rajan Cycle Collection (BiRC-safe) ──────────────────────────────
//
// Key design: we NEVER mutate local_rc or global_rc during cycle collection.
// Instead, we count "virtual decrements" in a HashMap. During scan, we check
// if (local_rc + global_rc) - virtual_decs <= 0 to determine garbage.
// This avoids the BiRC split-counter problem entirely and requires no STW.

fn collect_cycles(candidates: Vec<CircPtr>) {
    // Phase 0: Destroy objects that reached RC=0 but were deferred by circ_dec.
    // We must do this BEFORE cycle collection so their children's RCs are decremented.
    let mut actual_candidates = Vec::new();
    for &candidate_ptr in &candidates {
        let candidate = candidate_ptr.0;
        unsafe {
            if !candidate.is_null() {
                let header = &*candidate;
                let rc = total_rc(header);
                println!("phase_0: {:?}, color={}, rc={}", candidate, header.get_color(), rc);
                if rc == 0 {
                    println!("phase_0: freeing rc=0");
                    crate::circ::circ_destroy(candidate);
                } else {
                    println!("phase_0: pushing to actual_candidates");
                    actual_candidates.push(candidate_ptr);
                }
            }
        }
    }

    // Phase 1: Mark Gray — trace from PURPLE roots, count internal references in `crc`
    for &candidate_ptr in &actual_candidates {
        let candidate = candidate_ptr.0;
        unsafe {
            if !candidate.is_null() {
                let header = &*candidate;
                if header.get_color() == COLOR_PURPLE {
                    mark_gray(candidate);
                }
            }
        }
    }

    // Phase 2: Scan — check if objects are truly garbage
    for &candidate_ptr in &actual_candidates {
        let candidate = candidate_ptr.0;
        unsafe {
            if !candidate.is_null() {
                scan(candidate);
            }
        }
    }

    // Phase 3: Collect White — free garbage cycles
    for &candidate_ptr in &actual_candidates {
        let candidate = candidate_ptr.0;
        unsafe {
            if !candidate.is_null() {
                collect_white(candidate);
            }
        }
    }
}

/// Combined RC from both BiRC counters.
unsafe fn total_rc(header: &CircHeader) -> i64 {
    // global_rc can underflow to represent negative values if a background thread
    // releases a reference acquired by the owner thread. We must cast to i32 first.
    header.local_rc as i64 + (header.global_rc.load(std::sync::atomic::Ordering::Relaxed) as i32) as i64
}

unsafe fn invoke_trace_fns(header_ptr: *mut CircHeader, visitor: *const ()) {
    let obj_ptr = (header_ptr as *mut u8).add(CircHeader::SIZE);
    
    if let Some(t_fn) = get_trace_fn(header_ptr) {
        t_fn(obj_ptr, visitor);
    }
    
    
    
    // Trace dynamic properties
    {
        let map = crate::objects::dynamic_props::DYNAMIC_PROPERTIES.lock().unwrap();
        // println!("invoke_trace_fns for obj_ptr: {:?} (as usize: {})", obj_ptr, obj_ptr as usize);
        // print!("Keys in map: ");
        // for k in map.keys() {
        //     print!("{} ", k);
        // }
        // println!();
        if let Some(props) = map.get(&(obj_ptr as usize)) {
            // println!("invoke_trace_fns: found {} dynamic props for {:?}", props.len(), obj_ptr);
            for (_name, &val_tagged) in props.iter() {
                // println!("  prop {}: {:x}", name, val_tagged);
                let rc_tag = val_tagged >> 48;
                if rc_tag == 0xFFF6 || rc_tag == 0xFFF9 || rc_tag == 0xFFFA || rc_tag == 0xFFFB || rc_tag == 0x7FFE {
                    let unbox_ptr = val_tagged & 0x0000_FFFF_FFFF_FFFF;
                    let raw_ptr = unbox_ptr as *mut u8;
                    if !raw_ptr.is_null() {
                        let child = raw_ptr.sub(CircHeader::SIZE) as *mut CircHeader;
                        // // println!("  trace dynamic prop {}: {:?}", name, child);
                        let visitor_fn: unsafe extern "C-unwind" fn(*mut CircHeader) = std::mem::transmute(visitor);
                        visitor_fn(child);
                    }
                }
            }
        }
    }
    
    // Trace inline properties
    let flags = (*header_ptr).flags.load(std::sync::atomic::Ordering::Relaxed);
    if (flags & crate::circ::VTABLE_PTR) != 0 {
        let props_slot = obj_ptr.add(8) as *mut *mut std::collections::HashMap<String, u64>;
        if !(*props_slot).is_null() {
            let bx = Box::from_raw(*props_slot);
            for (_name, &val_tagged) in bx.iter() {
                let rc_tag = val_tagged >> 48;
                if rc_tag == 0xFFF6 || rc_tag == 0xFFF9 || rc_tag == 0xFFFA || rc_tag == 0xFFFB || rc_tag == 0x7FFE {
                    let unbox_ptr = val_tagged & 0x0000_FFFF_FFFF_FFFF;
                    let raw_ptr = unbox_ptr as *mut u8;
                    if !raw_ptr.is_null() {
                        let child = raw_ptr.sub(CircHeader::SIZE) as *mut CircHeader;
                        // println!("  trace inline prop {}: {:?}", name, child);
                        let visitor_fn: unsafe extern "C-unwind" fn(*mut CircHeader) = std::mem::transmute(visitor);
                        visitor_fn(child);
                    }
                }
            }
            let _ = Box::into_raw(bx);
        }
    }
    
}

// Bacon-Rajan Mark Gray: count virtual decrements using in-place `crc` field
unsafe fn mark_gray(s: *mut CircHeader) {
    if s.is_null() { return; }
    let header = &mut *s;
    let color = header.get_color();
    // println!("mark_gray: {:?}, color={}, crc={}, total_rc={}", s, color, header.crc, total_rc(header));
    if color != COLOR_GRAY {
        header.set_color(COLOR_GRAY);
        header.crc = 0; // Initialize when first visited

        invoke_trace_fns(s, mark_gray_visitor as *const ());
    }
}

unsafe extern "C-unwind" fn mark_gray_visitor(header_ptr: *mut CircHeader) {
    let header = &mut *header_ptr;
    let is_new = header.get_color() != COLOR_GRAY;
    if is_new {
        header.set_color(COLOR_GRAY);
        header.crc = 0;
    }
    
    // Count this as one internal reference to the child
    header.crc += 1;
    
    if is_new {
        invoke_trace_fns(header_ptr, mark_gray_visitor as *const ());
    }
}

fn get_trace_fn_class_name(header: *mut CircHeader) -> String {
    let flags = unsafe { (*header).flags.load(std::sync::atomic::Ordering::Relaxed) };
    if flags & crate::circ::VTABLE_PTR != 0 {
        unsafe {
            let obj_ptr = (header as *mut u8).add(CircHeader::SIZE);
            let vtable_ptr = *(obj_ptr as *mut *mut u8);
            if !vtable_ptr.is_null() {
                let name_ptr = *(vtable_ptr.add(8) as *mut *const u8);
                if !name_ptr.is_null() {
                    return std::ffi::CStr::from_ptr(name_ptr as *const i8).to_string_lossy().into_owned();
                }
            }
        }
    } else if flags & crate::circ::IS_CLOSURE != 0 {
        return "Closure".to_string();
    }
    "Unknown".to_string()
}

// Bacon-Rajan Scan: check (total_rc - crc) to determine liveness
unsafe fn scan(s: *mut CircHeader) {
    if s.is_null() { return; }
    let header = &mut *s;
    if header.get_color() == COLOR_GRAY {
        let rc = total_rc(header);
        let decs = header.crc as i64;
        let effective_rc = rc - decs;
        // // println!("scan: {:?}, rc={}, decs={}, eff={}", s, rc, decs, effective_rc);
        if effective_rc > 0 {
            scan_black(s);
        } else {
            header.set_color(COLOR_WHITE);
            // Trace children and scan them too
            invoke_trace_fns(s, scan_visitor as *const ());
        }
    }
}

unsafe extern "C-unwind" fn scan_visitor(header_ptr: *mut CircHeader) {
    scan(header_ptr);
}

unsafe fn scan_black(s: *mut CircHeader) {
    if s.is_null() { return; }
    let header = &mut *s;
    header.set_color(COLOR_BLACK);
    
    invoke_trace_fns(s, scan_black_visitor as *const ());
}

unsafe extern "C-unwind" fn scan_black_visitor(header_ptr: *mut CircHeader) {
    let header = &mut *header_ptr;
    header.crc -= 1;
    if header.get_color() != COLOR_BLACK {
        scan_black(header_ptr);
    }
}

// Bacon-Rajan Collect White: free the cycle
//
// Key insight: we must NOT call drop_fn on cycle members, because drop_fn
// calls circ_dec on children whose RC counts are for live objects.
// Instead, we recursively collect_white all children first (so they are freed
// from the leaves inward), then free the node's memory directly.

unsafe extern "C-unwind" fn decrement_black_visitor(child_header_ptr: *mut CircHeader) {
    if child_header_ptr.is_null() { return; }
    let child_header = &mut *child_header_ptr;
    let color = child_header.get_color();
    if color != COLOR_WHITE && color != crate::circ::COLOR_FREEING {
        crate::circ::circ_dec(child_header_ptr);
    }
}

unsafe fn collect_white(s: *mut CircHeader) {
    if s.is_null() { return; }
    let header = &*s;
    if header.get_color() == COLOR_WHITE && !header.is_buffered() {
        header.set_color(crate::circ::COLOR_FREEING); // prevent double-visit

        // First, decrement RC of external children before freeing this object.
        // We do this BEFORE recursing to collect_white_visitor, because if we
        // recurse first, the children might be freed, and then we'd read their
        // memory here to check their color, resulting in use-after-free.
        invoke_trace_fns(s, decrement_black_visitor as *const ());

        // Now recurse into white children so they are collected
        invoke_trace_fns(s, collect_white_visitor as *const ());

        let obj_ptr = (s as *mut u8).add(CircHeader::SIZE);
        
        // Free inline property map
        let flags = (*s).flags.load(std::sync::atomic::Ordering::Relaxed);
        if (flags & crate::circ::VTABLE_PTR) != 0 {
            let props_slot = obj_ptr.add(8) as *mut *mut std::collections::HashMap<String, u64>;
            crate::objects::dynamic_props::free_inline_properties_only(props_slot);
            
            // Map/Set/WeakMap/WeakSet have no trace_fn but have a drop_fn that MUST be called
            // to clean up external memory (MAP_DATA, SET_DATA).
            let vtable_ptr = *(obj_ptr as *mut *mut u8);
            if !vtable_ptr.is_null() {
                let drop_fn_ptr_addr = vtable_ptr.add(8 * 5); 
                let drop_fn_ptr = *(drop_fn_ptr_addr as *mut *const u8);
                let trace_fn_ptr_addr = vtable_ptr.add(8 * 6);
                let trace_fn_ptr = *(trace_fn_ptr_addr as *mut *const u8);
                
                // println!("collect_white: checking VTABLE_PTR drop={:?} trace={:?}", drop_fn_ptr, trace_fn_ptr);
                
                // If it has no trace_fn, its children are not in the cycle, so drop_fn is safe
                if !drop_fn_ptr.is_null() && trace_fn_ptr.is_null() {
                    // println!("collect_white: calling drop_fn for VTABLE_PTR {:?}", vtable_ptr);
                    let drop_fn: unsafe extern "C-unwind" fn(*mut u8) = std::mem::transmute(drop_fn_ptr);
                    drop_fn(obj_ptr);
                }
            }
        } else if (flags & crate::circ::IS_ARRAY) != 0 {
            crate::array::free_array_buffer_only(obj_ptr);
        }
        
        crate::objects::dynamic_props::remove_dynamic_properties_only(obj_ptr);
        crate::verify::__bs_verify_track_free(obj_ptr);
        crate::circ::SHARED_FREES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let size = (*s).alloc_size;
        crate::slab::fast_free_shared(s as *mut u8, size);
    }
}

unsafe extern "C-unwind" fn collect_white_visitor(header_ptr: *mut CircHeader) {
    collect_white(header_ptr);
}

// Helper to extract trace_fn from VTable
unsafe fn get_trace_fn(header: *mut CircHeader) -> Option<extern "C-unwind" fn(*mut u8, *const ())> {
    let flags = (*header).flags.load(std::sync::atomic::Ordering::Relaxed);
    if flags & crate::circ::VTABLE_PTR != 0 {
        let obj_ptr = (header as *mut u8).add(CircHeader::SIZE);
        let vtable_ptr_ptr = obj_ptr as *mut *mut u8;
        let vtable_ptr = *vtable_ptr_ptr;
        
        if !vtable_ptr.is_null() {
            // VTable struct: parent, name, shape_id, fields_count, field_names, drop_fn, trace_fn
            let trace_fn_ptr_addr = vtable_ptr.add(8 * 6); 
            let trace_fn_ptr = *(trace_fn_ptr_addr as *mut *const u8);
            if !trace_fn_ptr.is_null() {
                return Some(std::mem::transmute(trace_fn_ptr));
            }
        }
    } else if flags & crate::circ::IS_CLOSURE != 0 {
        let obj_ptr = (header as *mut u8).add(CircHeader::SIZE);
        
        let trace_fn_ptr_addr = obj_ptr.add(16);
        let trace_fn_ptr = *(trace_fn_ptr_addr as *mut *const u8);
        if !trace_fn_ptr.is_null() {
            return Some(std::mem::transmute(trace_fn_ptr));
        }
    } else if flags & crate::circ::IS_ARRAY != 0 {
        return Some(std::mem::transmute(crate::array::__bs_array_trace_elements as *const ()));
    } else if flags & crate::circ::IS_GENERATOR != 0 {
        let obj_ptr = (header as *mut u8).add(CircHeader::SIZE);
        
        let trace_fn_ptr_addr = obj_ptr.add(16);
        let trace_fn_ptr = *(trace_fn_ptr_addr as *mut *const u8);
        if !trace_fn_ptr.is_null() {
            return Some(std::mem::transmute(trace_fn_ptr));
        }
    }
    None
}
