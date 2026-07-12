use std::sync::atomic::*;
pub struct CircHeader {
    pub local_rc: u32,
    pub global_rc: AtomicI32,
    pub owner_tid: AtomicU32,
    pub flags: std::sync::atomic::AtomicU16,
    pub alloc_size: u16,
    pub crc: u32,
}
fn main() {
    println!("{}", std::mem::size_of::<CircHeader>());
}
