import os

with open("rt-stubs/src/dynamic_call/custom.rs", "r") as f:
    content = f.read()
# Replace any multiple occurrences
import re
content = re.sub(r'TAG_OBJECT,\s*(?:TAG_OWNED,\s*TAG_ARENA,\s*)+', 'TAG_OBJECT, TAG_OWNED, TAG_ARENA, ', content)
with open("rt-stubs/src/dynamic_call/custom.rs", "w") as f:
    f.write(content)

with open("rt-stubs/src/dynamic_call/dispatchers.rs", "r") as f:
    content = f.read()
content = content.replace("TAG_OBJECT, PAYLOAD_MASK", "TAG_OBJECT, TAG_OWNED, TAG_ARENA, PAYLOAD_MASK")
with open("rt-stubs/src/dynamic_call/dispatchers.rs", "w") as f:
    f.write(content)
