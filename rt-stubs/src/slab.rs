use std::alloc::{Layout, alloc};
use std::cell::Cell;

const CHUNK_SIZE: usize = 65536; // 64KB chunks

struct FreeList {
    head: Cell<*mut u8>,
    element_size: usize,
}

impl FreeList {
    const fn new(size: usize) -> Self {
        Self { head: Cell::new(std::ptr::null_mut()), element_size: size }
    }

    #[inline(always)]
    unsafe fn alloc(&self) -> *mut u8 {
        let mut ptr = self.head.get();
        if ptr.is_null() {
            self.refill();
            ptr = self.head.get();
            if ptr.is_null() {
                return std::ptr::null_mut();
            }
        }
        self.head.set(*(ptr as *mut *mut u8));
        ptr
    }

    #[inline(always)]
    unsafe fn free(&self, ptr: *mut u8) {
        *(ptr as *mut *mut u8) = self.head.get();
        self.head.set(ptr);
    }

    #[cold]
    unsafe fn refill(&self) {
        let layout = Layout::from_size_align(CHUNK_SIZE, 8).unwrap();
        let chunk = alloc(layout);
        if chunk.is_null() {
            return;
        }
        
        // Zero the chunk initially
        libc::memset(chunk as *mut libc::c_void, 0, CHUNK_SIZE);
        
        let count = CHUNK_SIZE / self.element_size;
        for i in 0..count {
            let ptr = chunk.add(i * self.element_size);
            if i < count - 1 {
                let next = chunk.add((i + 1) * self.element_size);
                *(ptr as *mut *mut u8) = next;
            } else {
                *(ptr as *mut *mut u8) = std::ptr::null_mut();
            }
        }
        self.head.set(chunk);
    }
}

thread_local! {
    static BIN_32: FreeList = FreeList::new(32);
    static BIN_64: FreeList = FreeList::new(64);
    static BIN_128: FreeList = FreeList::new(128);
    static BIN_256: FreeList = FreeList::new(256);
}

/// Fast thread-local allocation for CIRC objects.
/// Returns (pointer, alloc_size). alloc_size is 0 if allocated via libc.
#[inline(always)]
pub unsafe fn fast_alloc_shared(size: usize) -> (*mut u8, u16) {
    let ptr = if size <= 32 {
        BIN_32.with(|b| b.alloc())
    } else if size <= 64 {
        BIN_64.with(|b| b.alloc())
    } else if size <= 128 {
        BIN_128.with(|b| b.alloc())
    } else if size <= 256 {
        BIN_256.with(|b| b.alloc())
    } else {
        std::ptr::null_mut()
    };

    if !ptr.is_null() {
        let bin_size = if size <= 32 { 32 } else if size <= 64 { 64 } else if size <= 128 { 128 } else { 256 };
        // We must zero the memory as expected by posix_memalign and callers
        libc::memset(ptr as *mut libc::c_void, 0, bin_size);
        (ptr, bin_size as u16)
    } else {
        let mut raw = std::ptr::null_mut();
        if libc::posix_memalign(&mut raw, 8, size) != 0 {
            panic!("Out of memory");
        }
        libc::memset(raw, 0, size);
        (raw as *mut u8, 0)
    }
}

/// Fast thread-local deallocation for CIRC objects.
#[inline(always)]
pub unsafe fn fast_free_shared(ptr: *mut u8, alloc_size: u16) {
    if alloc_size == 32 {
        BIN_32.with(|b| b.free(ptr));
    } else if alloc_size == 64 {
        BIN_64.with(|b| b.free(ptr));
    } else if alloc_size == 128 {
        BIN_128.with(|b| b.free(ptr));
    } else if alloc_size == 256 {
        BIN_256.with(|b| b.free(ptr));
    } else {
        libc::free(ptr as *mut libc::c_void);
    }
}
