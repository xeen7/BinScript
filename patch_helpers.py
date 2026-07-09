with open("rt-stubs/src/dynamic_call/helpers.rs", "r") as f:
    code = f.read()

# Replace TAG_OWNED_CLOSURE
code = code.replace("pub const TAG_OWNED_CLOSURE: u64 = 0xFFF0_0000_0000_0000;", """pub const TAG_OWNED_CLOSURE: u64 = 0x7FF9_0000_0000_0000;
pub const TAG_OWNED_ARRAY: u64 = 0x7FFB_0000_0000_0000;
pub const TAG_OWNED_STRING: u64 = 0x7FF7_0000_0000_0000;
pub const TAG_ARENA_ARRAY: u64 = 0x7FFA_0000_0000_0000;""")

with open("rt-stubs/src/dynamic_call/helpers.rs", "w") as f:
    f.write(code)
