with open("crates/codegen-llvm/src/codegen/mod.rs", "r") as f:
    code = f.read()

code = code.replace("""        add(self, "__bs_alloc_array", self.i64_ty.fn_type(&[], false));""", """        add(self, "__bs_alloc_array", self.i64_ty.fn_type(&[], false));
        add(self, "__bs_alloc_owned_array", self.i64_ty.fn_type(&[], false));
        add(self, "__bs_alloc_arena_array", self.i64_ty.fn_type(&[self.i32_ty.into()], false));""")

with open("crates/codegen-llvm/src/codegen/mod.rs", "w") as f:
    f.write(code)
