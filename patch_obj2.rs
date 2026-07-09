use std::fs;

fn main() {
    let content = fs::read_to_string("rt-stubs/src/objects/builtins/object.rs").unwrap();
    let mut new_content = String::new();
    
    let is_obj = "
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
";

    let is_target_obj = "
        let is_target_object = if target_tag == 0xFFFC_0000_0000_0000 {
            let header = target_ptr.wrapping_sub(crate::circ::CircHeader::SIZE) as *const crate::circ::CircHeader;
            let flags = unsafe { (*header).flags.load(std::sync::atomic::Ordering::Relaxed) };
            (flags & crate::circ::VTABLE_PTR) != 0
        } else {
            true
        };
        if !is_target_object { return target; }
";

    let is_source_obj = "
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
";

    let mut in_object_block = false;
    let mut in_source_block = false;
    
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        
        if line.contains("if tag == 0xFFF6_0000_0000_0000 {") {
            new_content.push_str("    if tag == 0xFFF6_0000_0000_0000 || tag == 0xFFFC_0000_0000_0000 || tag == 0xFFFE_0000_0000_0000 {\n");
        } else if line.contains("if target_tag == 0xFFF6_0000_0000_0000 {") {
            new_content.push_str(is_target_obj);
            new_content.push_str("        if source_tag == 0xFFF6_0000_0000_0000 || source_tag == 0xFFFC_0000_0000_0000 || source_tag == 0xFFFE_0000_0000_0000 {\n");
        } else if line.contains("if source_tag == 0xFFF6_0000_0000_0000 {") {
            new_content.push_str("        if source_tag == 0xFFF6_0000_0000_0000 || source_tag == 0xFFFC_0000_0000_0000 || source_tag == 0xFFFE_0000_0000_0000 {\n");
        } else if line.contains("let vtable_ptr = *(obj_ptr as *const *const VTable);") {
            new_content.push_str(is_obj);
            new_content.push_str(line);
            new_content.push('\n');
            in_object_block = true;
        } else if line.contains("let vtable_ptr = *(source_ptr as *const *const VTable);") {
            new_content.push_str(is_source_obj);
            new_content.push_str(line);
            new_content.push('\n');
            in_source_block = true;
        } else if in_object_block && line == "        }" && lines[i+1] == "    }" {
            new_content.push_str("            }\n");
            new_content.push_str(line);
            new_content.push('\n');
            in_object_block = false;
        } else if in_source_block && line == "        }" && lines[i+1] == "    }" && lines.get(i+2) != Some(&"}") {
            // Need to close `is_source_object`
            new_content.push_str("            }\n");
            new_content.push_str(line);
            new_content.push('\n');
            in_source_block = false;
        } else {
            new_content.push_str(line);
            new_content.push('\n');
        }
        i += 1;
    }
    fs::write("rt-stubs/src/objects/builtins/object.rs", new_content).unwrap();
}
