import re

with open("rt-stubs/src/objects/builtins/object.rs", "r") as f:
    lines = f.readlines()

new_lines = []
for line in lines:
    if "if tag == 0xFFF6_0000_0000_0000 {" in line:
        new_lines.append(line.replace("if tag == 0xFFF6_0000_0000_0000 {", "if tag == 0xFFF6_0000_0000_0000 || tag == 0xFFFC_0000_0000_0000 || tag == 0xFFFE_0000_0000_0000 {"))
    elif "if target_tag == 0xFFF6_0000_0000_0000 {" in line:
        new_lines.append(line.replace("if target_tag == 0xFFF6_0000_0000_0000 {", "if target_tag == 0xFFF6_0000_0000_0000 || target_tag == 0xFFFC_0000_0000_0000 || target_tag == 0xFFFE_0000_0000_0000 {"))
    elif "if source_tag == 0xFFF6_0000_0000_0000 {" in line:
        new_lines.append(line.replace("if source_tag == 0xFFF6_0000_0000_0000 {", "if source_tag == 0xFFF6_0000_0000_0000 || source_tag == 0xFFFC_0000_0000_0000 || source_tag == 0xFFFE_0000_0000_0000 {"))
    elif "let vtable_ptr = *(obj_ptr as *const *const VTable);" in line:
        new_lines.append("""
            let is_object = if tag == 0xFFFC_0000_0000_0000 {
                let header = obj_ptr.wrapping_sub(crate::circ::CircHeader::SIZE) as *const crate::circ::CircHeader;
                let flags = unsafe { (*header).flags.load(std::sync::atomic::Ordering::Relaxed) };
                (flags & crate::circ::VTABLE_PTR) != 0
            } else {
                true
            };
            if !is_object {
                // Not an object
            } else {
""" + line)
    elif "let vtable_ptr = *(source_ptr as *const *const VTable);" in line:
        new_lines.append("""
            let is_source_object = if source_tag == 0xFFFC_0000_0000_0000 {
                let header = source_ptr.wrapping_sub(crate::circ::CircHeader::SIZE) as *const crate::circ::CircHeader;
                let flags = unsafe { (*header).flags.load(std::sync::atomic::Ordering::Relaxed) };
                (flags & crate::circ::VTABLE_PTR) != 0
            } else {
                true
            };
            if !is_source_object {
                // Not an object
            } else {
""" + line)
    elif "if source_tag == 0xFFF6_0000_0000_0000" in line:
        # Before this line, we need to check target_object
        new_lines.append("""
        let is_target_object = if target_tag == 0xFFFC_0000_0000_0000 {
            let header = target_ptr.wrapping_sub(crate::circ::CircHeader::SIZE) as *const crate::circ::CircHeader;
            let flags = unsafe { (*header).flags.load(std::sync::atomic::Ordering::Relaxed) };
            (flags & crate::circ::VTABLE_PTR) != 0
        } else {
            true
        };
        if !is_target_object { return target; }
""")
        new_lines.append(line.replace("if source_tag == 0xFFF6_0000_0000_0000 {", "if source_tag == 0xFFF6_0000_0000_0000 || source_tag == 0xFFFC_0000_0000_0000 || source_tag == 0xFFFE_0000_0000_0000 {"))
    else:
        new_lines.append(line)

# Now we have unclosed brackets. We need to find the end of the `if payload != 0` block and add a `}` for `if !is_object { } else {` block.
# Actually, the `is_object` block is replacing the `vtable_ptr` line. We opened `if !is_object {} else {`. This wraps the REST of the `if payload != 0` block.
# So we need to add a `}` right before the closing `}` of `if payload != 0`.

code = "".join(new_lines)
with open("rt-stubs/src/objects/builtins/object.rs.tmp", "w") as f:
    f.write(code)

