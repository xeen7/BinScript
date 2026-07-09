with open("crates/codegen-llvm/src/codegen/mod.rs", "r") as f:
    code = f.read()

# print array for TAG_OWNED_ARRAY and TAG_ARENA_ARRAY
code = code.replace("""                (self.i64_ty.const_int(0xFFFB, false), print_array),""", """                (self.i64_ty.const_int(0xFFFB, false), print_array),
                (self.i64_ty.const_int(0x7FFB, false), print_array),
                (self.i64_ty.const_int(0x7FFA, false), print_array),""")

# print string for TAG_OWNED_STRING
code = code.replace("""                (self.i64_ty.const_int(0xFFF7, false), print_str),""", """                (self.i64_ty.const_int(0xFFF7, false), print_str),
                (self.i64_ty.const_int(0x7FF7, false), print_str),""")

# print closure for TAG_OWNED_CLOSURE
code = code.replace("""                (self.i64_ty.const_int(0xFFF9, false), print_closure),
                (self.i64_ty.const_int(0xFFF0, false), print_closure),""", """                (self.i64_ty.const_int(0xFFF9, false), print_closure),
                (self.i64_ty.const_int(0x7FF9, false), print_closure),""")

with open("crates/codegen-llvm/src/codegen/mod.rs", "w") as f:
    f.write(code)
