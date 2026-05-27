use std::sync::Mutex;
use once_cell::sync::Lazy;

/// A header prepended to every object allocated on the GC heap.
#[repr(C, align(8))]
pub struct GcObject {
    pub marked: bool,
    pub size: u32,
    pub tag: u16,        // NaN-box tag (0xFFF6=object, 0xFFF9=closure, 0xFFFA=generator)
    pub num_slots: u16,  // Number of i64 slots that may contain GC pointers
    // Payload follows
}

pub struct GcHeap {
    objects: Vec<*mut GcObject>,
    pub bytes_allocated: usize,
    pub threshold: usize,
}

unsafe impl Send for GcHeap {}
unsafe impl Sync for GcHeap {}

static GC_HEAP: Lazy<Mutex<GcHeap>> = Lazy::new(|| {
    Mutex::new(GcHeap {
        objects: Vec::new(),
        bytes_allocated: 0,
        threshold: 1024 * 1024, // 1 MB
    })
});

/// Global roots for values captured by Promises and Microtasks.
/// In a real engine, closures would be fully introspectable. Here we just root them.
pub static GLOBAL_ROOTS: Lazy<Mutex<Vec<u64>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub unsafe fn gc_alloc(size: usize, tag: u16, num_slots: u16) -> *mut u8 {
    let mut heap = GC_HEAP.lock().unwrap();

    if heap.bytes_allocated >= heap.threshold {
        drop(heap); // release lock before collect
        gc_collect();
        heap = GC_HEAP.lock().unwrap();
        // Increase threshold to avoid excessive GC
        heap.threshold = std::cmp::max(heap.threshold, heap.bytes_allocated * 2);
    }

    let header_size = std::mem::size_of::<GcObject>();
    // Ensure header_size is a multiple of 8
    let header_size_aligned = (header_size + 7) & !7;
    let total_size = header_size_aligned + size;
    
    // Allocate aligned memory to prevent misaligned pointer dereference on the payload
    let mut ptr: *mut libc::c_void = std::ptr::null_mut();
    if libc::posix_memalign(&mut ptr, 8, total_size) != 0 {
        panic!("Out of memory");
    }
    let obj_ptr = ptr as *mut GcObject;

    libc::memset(ptr, 0, total_size);

    (*obj_ptr).marked = false;
    (*obj_ptr).size = total_size as u32;
    (*obj_ptr).tag = tag;
    (*obj_ptr).num_slots = num_slots;

    heap.objects.push(obj_ptr);
    heap.bytes_allocated += total_size;

    (ptr as *mut u8).add(header_size_aligned)
}

#[no_mangle]
pub unsafe extern "C" fn __bs_safepoint_poll() {
    let heap = GC_HEAP.lock().unwrap();
    let collect = heap.bytes_allocated >= heap.threshold;
    drop(heap);

    if collect {
        gc_collect();
    }
}

#[no_mangle]
pub unsafe extern "C" fn __bs_write_barrier(_parent: u64, _child: u64) {
    // No-op for mark-sweep. Future generational GC will track cross-generation pointers here.
}

pub unsafe fn gc_collect() {
    let mut heap = GC_HEAP.lock().unwrap();
    // println!("GC: Collecting... {} bytes allocated", heap.bytes_allocated);

    // 1. Mark phase
    // Scan shadow stack
    crate::shadow_stack::scan_roots(|val| {
        gc_mark_value(val);
    });

    // Scan global roots
    {
        let roots = GLOBAL_ROOTS.lock().unwrap();
        for &val in roots.iter() {
            gc_mark_value(val);
        }
    }

    // 2. Sweep phase
    let header_size = std::mem::size_of::<GcObject>();
    let header_size_aligned = (header_size + 7) & !7;
    let mut i = 0;
    while i < heap.objects.len() {
        let obj = heap.objects[i];
        if (*obj).marked {
            (*obj).marked = false; // reset for next GC
            i += 1;
        } else {
            let payload_ptr = (obj as *mut u8).add(header_size_aligned);
            crate::remove_dynamic_properties(payload_ptr);
            // Free array element buffer if this is an array
            if (*obj).tag == 0xFFFB {
                crate::array::free_array_data(payload_ptr);
            }
            
            heap.bytes_allocated -= (*obj).size as usize;
            libc::free(obj as *mut libc::c_void);
            heap.objects.swap_remove(i);
        }
    }
    // println!("GC: Finished. {} bytes allocated", heap.bytes_allocated);
}

pub unsafe fn gc_mark_value(val: u64) {
    let tag = (val & 0xFFFF_0000_0000_0000) >> 48;
    // 0xFFF6 = object, 0xFFF9 = closure, 0xFFFA = generator, 0xFFFB = array, 0xFFFC = promise
    if tag == 0xFFF6 || tag == 0xFFF9 || tag == 0xFFFA || tag == 0xFFFB || tag == 0xFFFC {
        let payload = val & 0x0000_FFFF_FFFF_FFFF;
        if payload == 0 {
            return;
        }
        let ptr = payload as *mut u8;
        let header = ptr.sub(std::mem::size_of::<GcObject>()) as *mut GcObject;
        
        if (*header).marked {
            return;
        }
        (*header).marked = true;

        // Array: trace element buffer instead of struct slots
        if tag == 0xFFFB {
            crate::array::trace_array(ptr);
            return;
        }

        // Scan slots
        let slots_ptr = ptr as *mut u64;
        let num_slots = (*header).num_slots as usize;
        for i in 0..num_slots {
            gc_mark_value(*slots_ptr.add(i));
        }

        // Trace dynamic properties for this object
        crate::trace_dynamic_properties(ptr);
    }
}

pub fn box_number(n: f64) -> u64 {
    let mut bits = n.to_bits();
    // If it's a NaN, canonicalize it to avoid colliding with our tags
    if bits & 0x7FF0_0000_0000_0000 == 0x7FF0_0000_0000_0000 && bits & 0x000F_FFFF_FFFF_FFFF != 0 {
        bits = 0x7FF8_0000_0000_0000;
    }
    bits
}

pub fn box_boolean(b: bool) -> u64 {
    if b {
        0xFFF4_0000_0000_0000
    } else {
        0xFFF3_0000_0000_0000
    }
}
