import os
import re

def process_file(path):
    with open(path, "r") as f:
        content = f.read()

    orig_content = content
    
    if path == "rt-stubs/src/dynamic_call/helpers.rs":
        if "TAG_OWNED" not in content:
            content = content.replace(
                "pub const TAG_OBJECT: u64 = 0xFFF6_0000_0000_0000;\n",
                "pub const TAG_OBJECT: u64 = 0xFFF6_0000_0000_0000;\npub const TAG_OWNED: u64 = 0xFFFC_0000_0000_0000;\npub const TAG_ARENA: u64 = 0xFFFE_0000_0000_0000;\n"
            )
    elif path in ["rt-stubs/src/dynamic_call/macros.rs", "rt-stubs/src/dynamic_call/custom.rs"]:
        if "TAG_OWNED" not in content:
            content = content.replace("TAG_OBJECT, PAYLOAD_MASK", "TAG_OBJECT, TAG_OWNED, TAG_ARENA, PAYLOAD_MASK")
            content = content.replace("TAG_OBJECT,", "TAG_OBJECT, TAG_OWNED, TAG_ARENA,")
            content = re.sub(r'tag == TAG_OBJECT', r'(tag == TAG_OBJECT || tag == TAG_OWNED || tag == TAG_ARENA)', content)
            content = re.sub(r'recv & TAG_MASK == TAG_OBJECT', r'(recv & TAG_MASK == TAG_OBJECT || recv & TAG_MASK == TAG_OWNED || recv & TAG_MASK == TAG_ARENA)', content)

    elif path != "rt-stubs/src/array/mod.rs":
        # Any occurrence of `var == 0xFFFC...` or `var != 0xFFFC...`
        content = re.sub(r'(\w+)\s*==\s*0xFFFC_0000_0000_0000', r'(\1 == 0xFFFC_0000_0000_0000 || \1 == 0xFFFE_0000_0000_0000)', content)
        content = re.sub(r'(\w+)\s*!=\s*0xFFFC_0000_0000_0000', r'(\1 != 0xFFFC_0000_0000_0000 && \1 != 0xFFFE_0000_0000_0000)', content)
    
    if content != orig_content:
        with open(path, "w") as f:
            f.write(content)

for root, _, files in os.walk("rt-stubs/src"):
    for file in files:
        if file.endswith(".rs"):
            process_file(os.path.join(root, file))

print("Done!")
