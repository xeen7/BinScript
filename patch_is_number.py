import os
import glob

def patch_file(filepath):
    with open(filepath, "r") as f:
        code = f.read()

    # The existing check is typically:
    # if tag == 0 || (tag > 0 && tag < 0xFFF0_0000_0000_0000) {
    # or
    # if tag < 0xFFF0_0000_0000_0000 {
    
    # Let's replace them with crate::dynamic_call::helpers::is_number(tag)
    # Actually, we need to pass the tag or val. Wait, `tag < 0xFFF0` assumes tag = val & TAG_MASK, which zeros the payload. But wait, `tag < 0xFFF0` is often used when `tag` is just `val & 0xFFFF_0000_0000_0000`.
    # Let's use crate::dynamic_call::helpers::is_number_tag(tag)
    
    code = code.replace("if tag == 0 || (tag > 0 && tag < 0xFFF0_0000_0000_0000) {", "if crate::dynamic_call::helpers::is_number_tag(tag) {")
    code = code.replace("} else if tag == 0 || (tag > 0 && tag < 0xFFF0_0000_0000_0000) {", "} else if crate::dynamic_call::helpers::is_number_tag(tag) {")
    code = code.replace("if tag < 0xFFF0_0000_0000_0000 {", "if crate::dynamic_call::helpers::is_number_tag(tag) {")

    if code != open(filepath, "r").read():
        with open(filepath, "w") as f:
            f.write(code)

for root, dirs, files in os.walk("rt-stubs"):
    for file in files:
        if file.endswith(".rs"):
            patch_file(os.path.join(root, file))

with open("rt-stubs/src/dynamic_call/helpers.rs", "a") as f:
    f.write("""
#[inline(always)]
pub fn is_number_tag(tag: u64) -> bool {
    let tag = tag & 0xFFFF_0000_0000_0000;
    if tag == 0x7FF7_0000_0000_0000 || tag == 0x7FF9_0000_0000_0000 || tag == 0x7FFA_0000_0000_0000 || tag == 0x7FFB_0000_0000_0000 {
        return false;
    }
    tag < 0xFFF0_0000_0000_0000
}
""")
