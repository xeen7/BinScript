#[repr(C, align(8))]
struct GcObject {
    marked: bool,
    size: u32,
    tag: u16,
    num_slots: u16,
}

fn main() {
    println!("sizeof GcObject = {}", std::mem::size_of::<GcObject>());
    println!("aligned = {}", (std::mem::size_of::<GcObject>() + 7) & !7);
}
