with open("rt-stubs/src/array/mod.rs", "r") as f:
    code = f.read()

code = code.replace("""pub unsafe extern "C-unwind" fn __bs_alloc_arena_array(region_id: u32) -> u64 {
    let raw = crate::core::arena::alloc_in_arena(region_id, std::mem::size_of::<BsArray>()) as *mut BsArray;""", """pub unsafe extern "C-unwind" fn __bs_alloc_arena_array(arena: *mut crate::arena::Arena) -> u64 {
    let raw = crate::arena::arena_alloc(arena, std::mem::size_of::<BsArray>(), 8) as *mut BsArray;""")

with open("rt-stubs/src/array/mod.rs", "w") as f:
    f.write(code)

with open("crates/codegen-llvm/src/codegen/mod.rs", "r") as f:
    code = f.read()

code = code.replace("""        add(self, "__bs_alloc_arena_array", self.i64_ty.fn_type(&[self.i32_ty.into()], false));""", """        add(self, "__bs_alloc_arena_array", self.i64_ty.fn_type(&[self.ptr_ty.into()], false));""")

with open("crates/codegen-llvm/src/codegen/mod.rs", "w") as f:
    f.write(code)

with open("crates/codegen-llvm/src/codegen/instr/objects/alloc_object.rs", "r") as f:
    code = f.read()

code = code.replace("""                if class_name == "Array" {
                    let alloc_fn = self.funcs["__bs_alloc_arena_array"];
                    let region_val = self.i32_ty.const_int(*region_id as u64, false);
                    let obj_val = self.builder.build_call(alloc_fn, &[region_val.into()], "alloc_arena_array").unwrap()""", """                if class_name == "Array" {
                    let arena_ptr = *self.arena_ptrs.get(region_id).unwrap();
                    let alloc_fn = self.funcs["__bs_alloc_arena_array"];
                    let obj_val = self.builder.build_call(alloc_fn, &[arena_ptr.into()], "alloc_arena_array").unwrap()""")

with open("crates/codegen-llvm/src/codegen/instr/objects/alloc_object.rs", "w") as f:
    f.write(code)
