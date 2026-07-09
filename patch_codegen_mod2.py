with open("crates/codegen-llvm/src/codegen/mod.rs", "r") as f:
    code = f.read()

code = code.replace("""        add(self, "__bs_string_concat", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));""", """        add(self, "__bs_string_concat", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        add(self, "__bs_string_concat_owned", self.i64_ty.fn_type(&[self.i64_ty.into(), self.i64_ty.into()], false));
        add(self, "__bs_number_to_string_owned", self.i64_ty.fn_type(&[self.i64_ty.into()], false));
        add(self, "__bs_boolean_to_string_owned", self.i64_ty.fn_type(&[self.i64_ty.into()], false));""")

with open("crates/codegen-llvm/src/codegen/mod.rs", "w") as f:
    f.write(code)
