use std::fs;

fn main() {
    let mut content = fs::read_to_string("crates/codegen-llvm/src/codegen/mod.rs").unwrap();
    let old = r#"                (self.i64_ty.const_int(0xFFF9, false), print_closure),
                (self.i64_ty.const_int(0xFFFC, false), print_obj),"#;
    let new = r#"                (self.i64_ty.const_int(0xFFF9, false), print_closure),
                (self.i64_ty.const_int(0xFFF0, false), print_closure),
                (self.i64_ty.const_int(0xFFFC, false), print_obj),
                (self.i64_ty.const_int(0xFFFE, false), print_obj),"#;
    content = content.replace(old, new);
    fs::write("crates/codegen-llvm/src/codegen/mod.rs", content).unwrap();
}
