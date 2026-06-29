

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_Array_new(val: u64) -> u64 {
    let tag = val & 0xFFFF_0000_0000_0000;
    if tag == 0xFFF1_0000_0000_0000 {
        crate::array::__bs_array_new()
    } else if tag == 0 || (tag > 0 && tag < 0xFFF0_0000_0000_0000) {
        let f = f64::from_bits(val);
        let len = f as u32;
        let tagged = crate::array::__bs_array_new();
        let arr = crate::array::untag_array(tagged);
        crate::array::grow_array(arr, len);
        (*arr).length = len;
        for i in 0..len {
            *(*arr).data.add(i as usize) = 0xFFF1_0000_0000_0000; // undefined
        }
        tagged
    } else {
        let tagged = crate::array::__bs_array_new();
        crate::array::__bs_array_push(tagged, val);
        tagged
    }
}
