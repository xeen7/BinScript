fn main() {
    let owned = 0xFFFC_0000_0000_0000u64;
    let shared = 0xFFF6_0000_0000_0000u64;
    let mask = 0xFFF5_0000_0000_0000u64;
    println!("Owned & mask = {:x}", owned & mask);
    println!("Shared & mask = {:x}", shared & mask);
}
