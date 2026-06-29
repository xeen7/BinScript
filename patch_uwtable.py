import re

content = open("crates/codegen-llvm/src/codegen/func.rs").read()

patch = """        // Give the function the uwtable attribute so it generates .eh_frame
        let uwtable_kind = inkwell::attributes::Attribute::get_named_enum_kind_id("uwtable");
        let uwtable_attr = self.ctx.create_enum_attribute(uwtable_kind, 2); // 2 means uwtable(async)
        fv.add_attribute(inkwell::attributes::AttributeLoc::Function, uwtable_attr);
"""

# add it after self.bbs.insert(b.id, bb); in emit_normal_function
content = re.sub(
    r'(for b in &func.blocks \{[^}]+\})',
    r'\1\n' + patch,
    content,
    count=1
)

open("crates/codegen-llvm/src/codegen/func.rs", "w").write(content)
