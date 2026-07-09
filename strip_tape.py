import re

with open("rt-stubs/src/json/tape.rs", "r") as f:
    code = f.read()

# Replace `is_object` checks in tape.rs
code = code.replace("""        let is_object = if tag == 0xFFF6_0000_0000_0000 || tag == 0xFFFE_0000_0000_0000 {
            true
        } else if tag == 0xFFFC_0000_0000_0000 {
            let header = obj_ptr.wrapping_sub(crate::circ::CircHeader::SIZE) as *const crate::circ::CircHeader;
            let flags = (*header).flags.load(std::sync::atomic::Ordering::Relaxed);
            (flags & crate::circ::VTABLE_PTR) != 0
        } else {
            false
        };

        if is_object {""", """        if true {""")

with open("rt-stubs/src/json/tape.rs", "w") as f:
    f.write(code)
