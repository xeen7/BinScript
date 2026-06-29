use std::mem;

#[repr(C, align(8))]
struct CircHeader {
    local_rc: u32,
    global_rc: i32,
    owner_tid: u32,
    flags: u16,
    alloc_size: u16,
    crc: u32,
}

fn main() {
    println!("CircHeader size: {}", mem::size_of::<CircHeader>());
    println!("CircHeader align: {}", mem::align_of::<CircHeader>());
}
