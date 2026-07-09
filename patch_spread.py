import re

with open("rt-stubs/src/objects/spread.rs", "r") as f:
    code = f.read()

# target_tag check
code = code.replace(
    "if target_tag != 0xFFF6_0000_0000_0000 {",
    "if target_tag != 0xFFF6_0000_0000_0000 && target_tag != 0xFFFC_0000_0000_0000 && target_tag != 0xFFFE_0000_0000_0000 {"
)

# is_target_object
code = code.replace(
    "let target_ptr = target_payload as *mut u8;",
    """let target_ptr = target_payload as *mut u8;
    let is_target_object = if target_tag == 0xFFFC_0000_0000_0000 {
        let header = target_ptr.wrapping_sub(crate::circ::CircHeader::SIZE) as *const crate::circ::CircHeader;
        let flags = unsafe { (*header).flags.load(std::sync::atomic::Ordering::Relaxed) };
        (flags & crate::circ::VTABLE_PTR) != 0
    } else {
        true
    };
    if !is_target_object {
        return target_tagged;
    }"""
)

# source_tag check
code = code.replace(
    "if source_tag == 0xFFF6_0000_0000_0000 {",
    "if source_tag == 0xFFF6_0000_0000_0000 || source_tag == 0xFFFC_0000_0000_0000 || source_tag == 0xFFFE_0000_0000_0000 {"
)

# is_source_object
code = code.replace(
    "            let src_ptr = src_payload as *mut u8;",
    """            let src_ptr = src_payload as *mut u8;
            let is_source_object = if source_tag == 0xFFFC_0000_0000_0000 {
                let header = src_ptr.wrapping_sub(crate::circ::CircHeader::SIZE) as *const crate::circ::CircHeader;
                let flags = unsafe { (*header).flags.load(std::sync::atomic::Ordering::Relaxed) };
                (flags & crate::circ::VTABLE_PTR) != 0
            } else {
                true
            };
            if is_source_object {"""
)

# Close is_source_object
code = code.replace(
    """            }
        }
    } else if source_tag == 0xFFF8_0000_0000_0000 { // TAG_JSON_TAPE""",
    """            }
            }
        }
    } else if source_tag == 0xFFF8_0000_0000_0000 { // TAG_JSON_TAPE"""
)


with open("rt-stubs/src/objects/spread.rs", "w") as f:
    f.write(code)
