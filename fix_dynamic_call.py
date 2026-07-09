import os

for path in ["rt-stubs/src/dynamic_call/macros.rs", "rt-stubs/src/dynamic_call/custom.rs"]:
    with open(path, "r") as f:
        content = f.read()
        
    # we need to import TAG_OWNED, TAG_ARENA if they are not already imported
    if "TAG_OBJECT" in content and "TAG_OWNED" not in content:
        content = content.replace("TAG_OBJECT, PAYLOAD_MASK", "TAG_OBJECT, TAG_OWNED, TAG_ARENA, PAYLOAD_MASK")
        content = content.replace("TAG_OBJECT,", "TAG_OBJECT, TAG_OWNED, TAG_ARENA,")
        
    content = content.replace("tag == TAG_OBJECT", "(tag == TAG_OBJECT || tag == TAG_OWNED || tag == TAG_ARENA)")
    content = content.replace("recv & TAG_MASK == TAG_OBJECT", "(recv & TAG_MASK == TAG_OBJECT || recv & TAG_MASK == TAG_OWNED || recv & TAG_MASK == TAG_ARENA)")
    
    with open(path, "w") as f:
        f.write(content)

print("Done fixing dynamic calls!")
