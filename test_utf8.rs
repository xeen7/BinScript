fn main() {
    let err = std::str::from_utf8(b"my-first-b\xFF").unwrap_err();
    println!("{:?}", err);
}
