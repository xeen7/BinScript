use std::fs;

fn main() {
    let content = fs::read_to_string("rt-stubs/src/objects/builtins/object.rs").unwrap();
    let mut new_content = String::new();
    let mut in_object_block = false;
    let mut in_target_block = false;
    let mut in_source_block = false;
    
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let mut line = lines[i];
        
        if line.contains("if tag == 0xFFF6_0000_0000_0000 {") {
            new_content.push_str("    if tag == 0xFFF6_0000_0000_0000 || tag == 0xFFFC_0000_0000_0000 || tag == 0xFFFE_0000_0000_0000 {\n");
        } else if line.contains("if target_tag == 0xFFF6_0000_0000_0000 {") {
            new_content.push_str("        let is_target_object = if target_tag == 0xFFFC_0000_0000_0000 {\n");
            new_content.push_str("            let header = target_ptr.wrapping_sub(crate::circ::CircHeader::SIZE) as *const crate::circ::CircHeader;\n");
            new_content.push_str("            let flags = unsafe { (*header).flags.load(std::sync::atomic::Ordering::Relaxed) };\n");
            new_content.push_str("            (flags & crate::circ::VTABLE_PTR) != 0\n");
            new_content.push_str("        } else {\n");
            new_content.push_str("            true\n");
            new_content.push_str("        };\n");
            new_content.push_str("        if !is_target_object { return target; }\n");
            new_content.push_str("        if source_tag == 0xFFF6_0000_0000_0000 || source_tag == 0xFFFC_0000_0000_0000 || source_tag == 0xFFFE_0000_0000_0000 {\n");
        } else if line.contains("if source_tag == 0xFFF6_0000_0000_0000 {") {
            new_content.push_str("        if source_tag == 0xFFF6_0000_0000_0000 || source_tag == 0xFFFC_0000_0000_0000 || source_tag == 0xFFFE_0000_0000_0000 {\n");
        } else if line.contains("let vtable_ptr = *(obj_ptr as *const *const VTable);") {
            new_content.push_str("            let is_object = if tag == 0xFFFC_0000_0000_0000 {\n");
            new_content.push_str("                let header = obj_ptr.wrapping_sub(crate::circ::CircHeader::SIZE) as *const crate::circ::CircHeader;\n");
            new_content.push_str("                let flags = unsafe { (*header).flags.load(std::sync::atomic::Ordering::Relaxed) };\n");
            new_content.push_str("                (flags & crate::circ::VTABLE_PTR) != 0\n");
            new_content.push_str("            } else {\n");
            new_content.push_str("                true\n");
            new_content.push_str("            };\n");
            new_content.push_str("            if is_object {\n");
            new_content.push_str(line);
            new_content.push('\n');
            in_object_block = true;
        } else if line.contains("let vtable_ptr = *(source_ptr as *const *const VTable);") {
            new_content.push_str("            let is_source_object = if source_tag == 0xFFFC_0000_0000_0000 {\n");
            new_content.push_str("                let header = source_ptr.wrapping_sub(crate::circ::CircHeader::SIZE) as *const crate::circ::CircHeader;\n");
            new_content.push_str("                let flags = unsafe { (*header).flags.load(std::sync::atomic::Ordering::Relaxed) };\n");
            new_content.push_str("                (flags & crate::circ::VTABLE_PTR) != 0\n");
            new_content.push_str("            } else {\n");
            new_content.push_str("                true\n");
            new_content.push_str("            };\n");
            new_content.push_str("            if is_source_object {\n");
            new_content.push_str(line);
            new_content.push('\n');
            in_source_block = true;
        } else if in_object_block && line == "        }" && lines[i+1] == "    }" && lines.get(i+2).map(|s| *s) == Some("") {
            new_content.push_str("            }\n");
            new_content.push_str(line);
            new_content.push('\n');
            in_object_block = false;
        } else if in_object_block && line == "        }" && lines[i+1] == "    }" && lines.get(i+2).map(|s| *s) == Some("    array") {
            new_content.push_str("            }\n");
            new_content.push_str(line);
            new_content.push('\n');
            in_object_block = false;
        } else if in_source_block && line == "        }" && lines[i+1] == "    }" && lines.get(i+2).map(|s| *s) == Some("    target") {
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
