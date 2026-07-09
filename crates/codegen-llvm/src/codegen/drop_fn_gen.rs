
use diagnostics::CompileResult;

use crate::codegen::LlvmCodegen;

impl<'ctx> LlvmCodegen<'ctx> {
    pub(super) fn generate_drop_fns(&mut self) -> CompileResult<()> {
        let class_names: Vec<String> = self.classes.keys().cloned().collect();

        for class_name in class_names {
            let fields_count = self.get_all_fields_count(&class_name);
            
            // For now, only generate drop_fn if the class has fields that need to be dropped
            // Later we will also check for RAII_EXTERNAL flag.
            if fields_count == 0 {
                continue;
            }

            let fn_name = format!("__bs_class_{}_drop_fn", class_name);
            let fn_type = self.void_ty.fn_type(&[self.ptr_ty.into()], false);
            let func = self.module.add_function(&fn_name, fn_type, None);
            
            // Store it in the map before building so we can reference it
            self.drop_fns.insert(class_name.clone(), func);

            let basic_block = self.ctx.append_basic_block(func, "entry");
            self.builder.position_at_end(basic_block);

            let obj_ptr = func.get_nth_param(0).unwrap().into_pointer_value();

            // Iterate through fields in reverse declaration order
            for i in (0..fields_count).rev() {
                // Field offset: vtable_ptr (8 bytes) + __mark (8 bytes) + index * 8 bytes
                let offset = self.i32_ty.const_int(16 + (i as u64) * 8, false);
                let field_ptr_ptr = unsafe {
                    self.builder.build_in_bounds_gep(self.i8_ty, obj_ptr, &[offset], &format!("field_ptr_ptr_{}", i)).unwrap()
                };

                let field_val = self.builder.build_load(self.i64_ty, field_ptr_ptr, &format!("field_val_{}", i)).unwrap().into_int_value();

                // Check if it's a heap pointer (NaN-boxed with specific bits).
                // Actually, the easiest way is to unbox and see if it's valid, but it might be a primitive.
                // Binscript NaN boxing: primitives are NaN, pointers are NaN-boxed.
                // A valid heap pointer in BinScript has top 16 bits as 0x0000 or 0xFFFF (if we use standard x86-64 pointers).
                // Wait, in circ.rs:
                // let mask = self.i64_ty.const_int(0x0000_FFFF_FFFF_FFFF, false);
                // let raw_ptr_i64 = self.builder.build_and(val, mask, "unbox_ptr").unwrap();
                // This assumes the value IS a pointer. If it's a number, it will be garbage.
                // We must check if the type tag is a heap object tag!
                // Let's use `nan.is_heap_object()` if it exists, or check the tag.
                
                // Let's call a helper `emit_circ_dec_if_heap_object`
                self.emit_circ_dec_if_heap_object(field_val);
            }

            self.builder.build_return(None).unwrap();
        }

        Ok(())
    }

    fn emit_circ_dec_if_heap_object(&mut self, val: inkwell::values::IntValue<'ctx>) {
        // We check if the JS value is any managed pointer (Shared or Owned)
        let is_obj = self.nan.is_any_managed_pointer(&self.builder, val);
        
        let current_fn = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let dec_block = self.ctx.append_basic_block(current_fn, "do_dec");
        let cont_block = self.ctx.append_basic_block(current_fn, "cont_dec");
        
        self.builder.build_conditional_branch(is_obj, dec_block, cont_block).unwrap();
        
        self.builder.position_at_end(dec_block);
        
        // Pass the raw tagged i64 value directly to __bs_cleanup_tagged
        let cleanup_fn = self.funcs["__bs_cleanup_tagged"];
        self.builder.build_call(cleanup_fn, &[val.into()], "call_cleanup").unwrap();
        
        self.builder.build_unconditional_branch(cont_block).unwrap();
        
        self.builder.position_at_end(cont_block);
    }
}
