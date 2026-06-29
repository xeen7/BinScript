extern "C" {
    fn __cxa_allocate_exception(thrown_size: usize) -> *mut u8;
    fn __cxa_throw(thrown_exception: *mut u8, tinfo: *mut u8, dest: *mut u8) -> !;
}
fn main() {
    unsafe {
        let ptr = __cxa_allocate_exception(8);
        *(ptr as *mut u64) = 42;
        __cxa_throw(ptr, std::ptr::null_mut(), std::ptr::null_mut());
    }
}
