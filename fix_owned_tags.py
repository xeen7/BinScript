import os

def process_file(path, replacements):
    with open(path, "r") as f:
        content = f.read()
    for old, new in replacements:
        content = content.replace(old, new)
    with open(path, "w") as f:
        f.write(content)

process_file("rt-stubs/src/dynamic_call/macros.rs", [
    ("let mut tag = recv & TAG_MASK;", "let mut tag = (recv | 0x8000_0000_0000_0000) & TAG_MASK;"),
])

process_file("rt-stubs/src/array/mod.rs", [
    ("let tag = val & TAG_MASK;", "let tag = (val | 0x8000_0000_0000_0000) & TAG_MASK;"),
    ("let tag = val & 0xFFFF_0000_0000_0000;", "let tag = (val | 0x8000_0000_0000_0000) & 0xFFFF_0000_0000_0000;"),
])

process_file("rt-stubs/src/types/coercion.rs", [
    ("let tag = val & 0xFFFF_0000_0000_0000;", "let tag = (val | 0x8000_0000_0000_0000) & 0xFFFF_0000_0000_0000;"),
])

process_file("rt-stubs/src/circ.rs", [
    ("let rc_tag = val >> 48;", "let rc_tag = (val | 0x8000_0000_0000_0000) >> 48;"),
    ("let rc_tag = captured_tagged >> 48;", "let rc_tag = (captured_tagged | 0x8000_0000_0000_0000) >> 48;"),
])

print("Done fixing owned tags!")
