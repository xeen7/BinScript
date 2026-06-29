use diagnostics::CompileResult;

use crate::codegen::LlvmCodegen;

impl<'ctx> LlvmCodegen<'ctx> {
    pub(super) fn generate_trace_fns(&mut self) -> CompileResult<()> {
        let class_names: Vec<String> = self.classes.keys().cloned().collect();

        for class_name in class_names {
            let fields_count = self.get_all_fields_count(&class_name);
            
            // For now, only generate trace_fn if the class has fields that need to be traced
            if fields_count == 0 {
                continue;
            }

            let fn_name = format!("__bs_class_{}_trace_fn", class_name);
            let fn_type = self.void_ty.fn_type(&[self.ptr_ty.into(), self.ptr_ty.into()], false);
            let func = self.module.add_function(&fn_name, fn_type, None);
            
            // Store it in the map before building so we can reference it
            self.trace_fns.insert(class_name.clone(), func);

            let basic_block = self.ctx.append_basic_block(func, "entry");
            self.builder.position_at_end(basic_block);

            let obj_ptr = func.get_nth_param(0).unwrap().into_pointer_value();
            let visitor_ptr = func.get_nth_param(1).unwrap().into_pointer_value();

            // Iterate through fields in reverse declaration order (same as drop_fn)
            for i in (0..fields_count).rev() {
                // Field offset: vtable_ptr (8 bytes) + __mark (8 bytes) + index * 8 bytes
                let offset = self.i32_ty.const_int(16 + (i as u64) * 8, false);
                let field_ptr_ptr = unsafe {
                    self.builder.build_in_bounds_gep(self.i8_ty, obj_ptr, &[offset], &format!("field_ptr_ptr_{}", i)).unwrap()
                };

                let field_val = self.builder.build_load(self.i64_ty, field_ptr_ptr, &format!("field_val_{}", i)).unwrap().into_int_value();
                
                self.emit_trace_visitor_if_heap_object(field_val, visitor_ptr);
            }

            self.builder.build_return(None).unwrap();
        }

        Ok(())
    }

    pub(super) fn emit_trace_visitor_if_heap_object(&mut self, val: inkwell::values::IntValue<'ctx>, visitor_ptr: inkwell::values::PointerValue<'ctx>) {
        let is_obj = self.nan.is_heap_pointer(&self.builder, val);
        
        let current_fn = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let trace_block = self.ctx.append_basic_block(current_fn, "do_trace");
        let cont_block = self.ctx.append_basic_block(current_fn, "cont_trace");
        
        self.builder.build_conditional_branch(is_obj, trace_block, cont_block).unwrap();
        
        self.builder.position_at_end(trace_block);
        
        // Unbox and call visitor
        let mask = self.i64_ty.const_int(0x0000_FFFF_FFFF_FFFF, false);
        let raw_ptr_i64 = self.builder.build_and(val, mask, "unbox_ptr").unwrap();
        let raw_ptr = self.builder.build_int_to_ptr(raw_ptr_i64, self.ptr_ty, "ptr").unwrap();
        
        let offset = self.i32_ty.const_int(24_u64.wrapping_neg(), true);
        let header_ptr = unsafe { self.builder.build_in_bounds_gep(self.i8_ty, raw_ptr, &[offset], "header_ptr").unwrap() };
        
        let visitor_fn_ty = self.void_ty.fn_type(&[self.ptr_ty.into()], false);
        self.builder.build_indirect_call(visitor_fn_ty, visitor_ptr, &[header_ptr.into()], "call_visitor").unwrap();
        
        self.builder.build_unconditional_branch(cont_block).unwrap();
        
        self.builder.position_at_end(cont_block);
    }
}
