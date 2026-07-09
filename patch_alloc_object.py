import re

with open("crates/codegen-llvm/src/codegen/instr/objects/alloc_object.rs", "r") as f:
    code = f.read()

# For Alloc/AllocShared:
alloc_shared = """            MirInstr::Alloc(dest, class_name) | MirInstr::AllocShared(dest, class_name) => {
                if class_name == "Array" {
                    let alloc_fn = self.funcs["__bs_alloc_array"];
                    let obj_val = self.builder.build_call(alloc_fn, &[], "alloc_array").unwrap()
                        .try_as_basic_value()
                        .basic()
                        .unwrap()
                        .into_int_value();
                    self.store(*dest, obj_val);
                    return Ok(());
                }
                let (size_in_bytes, vtable_ptr) = if class_name == "Object" {"""

code = code.replace("""            MirInstr::Alloc(dest, class_name) | MirInstr::AllocShared(dest, class_name) => {
                let (size_in_bytes, vtable_ptr) = if class_name == "Object" {""", alloc_shared)


alloc_owned = """            MirInstr::AllocOwned(dest, class_name) => {
                if class_name == "Array" {
                    let alloc_fn = self.funcs["__bs_alloc_owned_array"];
                    let obj_val = self.builder.build_call(alloc_fn, &[], "alloc_owned_array").unwrap()
                        .try_as_basic_value()
                        .basic()
                        .unwrap()
                        .into_int_value();
                    self.store(*dest, obj_val);
                    return Ok(());
                }
                let (size_in_bytes, vtable_ptr) = if class_name == "Object" {"""

code = code.replace("""            MirInstr::AllocOwned(dest, class_name) => {
                let (size_in_bytes, vtable_ptr) = if class_name == "Object" {""", alloc_owned)


alloc_arena = """            MirInstr::AllocArena(dest, class_name, region_id) => {
                if class_name == "Array" {
                    let alloc_fn = self.funcs["__bs_alloc_arena_array"];
                    let region_val = self.i32_ty.const_int(*region_id as u64, false);
                    let obj_val = self.builder.build_call(alloc_fn, &[region_val.into()], "alloc_arena_array").unwrap()
                        .try_as_basic_value()
                        .basic()
                        .unwrap()
                        .into_int_value();
                    self.store(*dest, obj_val);
                    return Ok(());
                }
                let (size_in_bytes, vtable_ptr) = if class_name == "Object" {"""

code = code.replace("""            MirInstr::AllocArena(dest, class_name, region_id) => {
                let (size_in_bytes, vtable_ptr) = if class_name == "Object" {""", alloc_arena)

with open("crates/codegen-llvm/src/codegen/instr/objects/alloc_object.rs", "w") as f:
    f.write(code)
