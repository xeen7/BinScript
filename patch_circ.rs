use std::fs;
fn main() {
    let mut content = fs::read_to_string("crates/codegen-llvm/src/codegen/instr/memory/circ.rs").unwrap();
    let old = r#"            let tag_owned = self.i64_ty.const_int(0xFFFC, false);
            let is_owned = self.builder.build_int_compare(IntPredicate::EQ, tag, tag_owned, "is_owned").unwrap();"#;
    let new = r#"            let tag_owned = self.i64_ty.const_int(0xFFFC, false);
            let tag_owned_closure = self.i64_ty.const_int(0xFFF0, false);
            let is_owned_obj = self.builder.build_int_compare(IntPredicate::EQ, tag, tag_owned, "is_owned_obj").unwrap();
            let is_owned_clo = self.builder.build_int_compare(IntPredicate::EQ, tag, tag_owned_closure, "is_owned_clo").unwrap();
            let is_owned = self.builder.build_or(is_owned_obj, is_owned_clo, "is_owned").unwrap();"#;
    content = content.replace(old, new);
    fs::write("crates/codegen-llvm/src/codegen/instr/memory/circ.rs", content).unwrap();
}
