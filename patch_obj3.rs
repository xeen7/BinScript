use std::fs;

fn main() {
    let content = fs::read_to_string("rt-stubs/src/objects/builtins/object.rs").unwrap();
    let mut new_content = String::new();
    
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

    let mut skip_next = false;
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if skip_next {
            skip_next = false;
            i += 1;
            continue;
        }
        
        if line.contains("let is_target_object = if target_tag == 0xFFFC_0000_0000_0000 {") {
            // Skip the next 8 lines
            i += 9;
            continue;
        }
        
        if line.contains("let target_ptr = target_payload as *mut u8;") {
            new_content.push_str(line);
            new_content.push('\n');
            new_content.push_str(is_target_obj);
        } else {
            new_content.push_str(line);
            new_content.push('\n');
        }
        i += 1;
    }
    fs::write("rt-stubs/src/objects/builtins/object.rs", new_content).unwrap();
}
