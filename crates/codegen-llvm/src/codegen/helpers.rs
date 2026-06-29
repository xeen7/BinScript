use std::collections::HashMap;


use inkwell::FloatPredicate;
use mir::types::*;
use diagnostics::{CompileError, CompileResult};

use crate::codegen::LlvmCodegen;

impl<'ctx> LlvmCodegen<'ctx> {
    // ── static global vtables ──────────────────────────────────────────────

    pub(super) fn emit_vtables(&mut self, mir: &MirModule) -> CompileResult<()> {
        let mut method_names: Vec<String> = mir.classes.values()
            .flat_map(|c| c.methods.iter().map(|m| m.name.clone()))
            .collect();
        method_names.sort();
        method_names.dedup();

        let mut class_names: Vec<String> = mir.classes.keys().cloned().collect();
        class_names.sort();
        let class_shapes: HashMap<String, u64> = class_names
            .iter()
            .enumerate()
            .map(|(i, name)| (name.clone(), (i + 1) as u64))
            .collect();

        // Structural type: { parent: ptr, name: ptr, shape_id: i64, fields_count: i64, field_names: ptr, slots: [num_methods x ptr] }
        let mut vtable_fields = vec![
            self.ptr_ty.into(), // parent
            self.ptr_ty.into(), // name
            self.i64_ty.into(), // shape_id
            self.i64_ty.into(), // fields_count
            self.ptr_ty.into(), // field_names
            self.ptr_ty.into(), // drop_fn
            self.ptr_ty.into(), // trace_fn
        ];
        for _ in 0..method_names.len() {
            vtable_fields.push(self.ptr_ty.into());
        }
        let vtable_ty = self.ctx.struct_type(&vtable_fields, false);

        // First, declare all vtable globals.
        for class_name in &class_names {
            let g = self.module.add_global(vtable_ty, None, &format!("__bs_class_{}_vtable", class_name));
            self.vtables.insert(class_name.clone(), g);
        }

        // Initialize each vtable.
        for class_name in &class_names {
            let class = &mir.classes[class_name];
            let shape_id = class_shapes[class_name];
            let g = self.vtables[class_name];

            let parent_val = if let Some(ref super_name) = class.super_name {
                let super_g = self.vtables[super_name];
                super_g.as_pointer_value()
            } else {
                self.ptr_ty.const_null()
            };

            let name_val = self.make_global_str(class_name);
            let shape_val = self.i64_ty.const_int(shape_id, false);

            // Build the static global array of field names for this class
            let fields = &class.fields;
            let fields_count_val = self.i64_ty.const_int(fields.len() as u64, false);
            let field_names_array_val = if fields.is_empty() {
                self.ptr_ty.const_null()
            } else {
                let mut field_ptrs = Vec::new();
                for (f, _) in fields {
                    field_ptrs.push(self.make_global_str(f).into());
                }
                let array_ty = self.ptr_ty.array_type(fields.len() as u32);
                let array_const = self.ptr_ty.const_array(&field_ptrs);
                let array_global = self.module.add_global(array_ty, None, &format!("__bs_class_{}_field_names", class_name));
                array_global.set_initializer(&array_const);
                array_global.set_constant(true);
                array_global.as_pointer_value()
            };

            let drop_fn_val = if let Some(drop_fn_func) = self.drop_fns.get(class_name) {
                drop_fn_func.as_global_value().as_pointer_value()
            } else {
                self.ptr_ty.const_null()
            };

            let trace_fn_val = if let Some(trace_fn_func) = self.trace_fns.get(class_name) {
                trace_fn_func.as_global_value().as_pointer_value()
            } else {
                self.ptr_ty.const_null()
            };

            let mut vals = vec![
                parent_val.into(),
                name_val.into(),
                shape_val.into(),
                fields_count_val.into(),
                field_names_array_val.into(),
                drop_fn_val.into(),
                trace_fn_val.into(),
            ];

            for m_name in &method_names {
                let slot_val = if let Some(impl_class) = self.find_method_impl(mir, class_name, m_name) {
                    let fn_name = format!("__bs_class_{}_{}", impl_class, m_name);
                    let fv = self.funcs.get(&fn_name).cloned().ok_or_else(|| {
                        CompileError::Codegen {
                            message: format!("Method implementation function {} not found", fn_name),
                        }
                    })?;
                    fv.as_global_value().as_pointer_value()
                } else {
                    self.ptr_ty.const_null()
                };
                vals.push(slot_val.into());
            }

            let init = self.ctx.const_struct(&vals, false);
            g.set_initializer(&init);
            g.set_constant(true);
        }

        Ok(())
    }

    fn find_method_impl(&self, mir: &MirModule, class_name: &str, method_name: &str) -> Option<String> {
        let mut curr = class_name;
        while let Some(class) = mir.classes.get(curr) {
            if class.methods.iter().any(|m| m.name == method_name) {
                return Some(curr.to_string());
            }
            if let Some(ref super_name) = class.super_name {
                curr = super_name;
            } else {
                break;
            }
        }
        None
    }

    pub(super) fn get_all_fields_count(&self, class_name: &str) -> usize {
        let mut count = 0;
        if let Some(class) = self.classes.get(class_name) {
            if let Some(ref super_name) = class.super_name {
                count += self.get_all_fields_count(super_name);
            }
            count += class.fields.len();
        }
        count
    }

    // ── operators & arithmetic helpers ──────────────────────────────────────

    pub(super) fn emit_add(
        &mut self,
        dest: MirReg,
        l: &MirOperand,
        r: &MirOperand,
    ) -> CompileResult<()> {
        let lv = self.val(l)?;
        let rv = self.val(r)?;
        let func = self.module.get_function("__bs_add").ok_or_else(|| CompileError::Codegen {
            message: "runtime helper __bs_add not found".to_string(),
        })?;
        let call = self.builder.build_call(func, &[lv.into(), rv.into()], "add_call").unwrap();
        let res = call.try_as_basic_value().basic().unwrap().into_int_value();
        self.store(dest, res);
        Ok(())
    }

    pub(super) fn emit_arith_f64(
        &mut self,
        dest: MirReg,
        l: &MirOperand,
        r: &MirOperand,
        op: &str,
    ) -> CompileResult<()> {
        let lv = self.val(l)?;
        let rv = self.val(r)?;
        let lf = self.nan.unbox_number(&self.builder, lv);
        let rf = self.nan.unbox_number(&self.builder, rv);
        let res = match op {
            "fadd" => self.builder.build_float_add(lf, rf, "add").unwrap(),
            "fsub" => self.builder.build_float_sub(lf, rf, "sub").unwrap(),
            "fmul" => self.builder.build_float_mul(lf, rf, "mul").unwrap(),
            "fdiv" => self.builder.build_float_div(lf, rf, "div").unwrap(),
            "frem" => self.builder.build_float_rem(lf, rf, "rem").unwrap(),
            _ => unreachable!(),
        };
        self.store(dest, self.nan.box_number(&self.builder, res));
        Ok(())
    }

    /// Emit a bitwise binary op: f64 → i32, integer op, i32 → f64, re-box.
    pub(super) fn emit_bitwise_i32(
        &mut self,
        dest: MirReg,
        l: &MirOperand,
        r: &MirOperand,
        op: &str,
    ) -> CompileResult<()> {
        let lv = self.val(l)?;
        let rv = self.val(r)?;
        let lf = self.nan.unbox_number(&self.builder, lv);
        let rf = self.nan.unbox_number(&self.builder, rv);
        let li = self.builder.build_float_to_signed_int(lf, self.i32_ty, "ltoi32").unwrap();
        let ri = self.builder.build_float_to_signed_int(rf, self.i32_ty, "rtoi32").unwrap();
        let res_i = match op {
            "and"  => self.builder.build_and(li, ri, "bitand").unwrap(),
            "or"   => self.builder.build_or(li, ri, "bitor").unwrap(),
            "xor"  => self.builder.build_xor(li, ri, "bitxor").unwrap(),
            "shl"  => {
                // Mask shift amount to 0..31 per JS spec
                let mask = self.i32_ty.const_int(0x1f, false);
                let shift = self.builder.build_and(ri, mask, "shlmask").unwrap();
                self.builder.build_left_shift(li, shift, "shl").unwrap()
            }
            "ashr" => {
                let mask = self.i32_ty.const_int(0x1f, false);
                let shift = self.builder.build_and(ri, mask, "shrmask").unwrap();
                self.builder.build_right_shift(li, shift, true, "ashr").unwrap()
            }
            _ => unreachable!(),
        };
        let f64_ty = self.ctx.f64_type();
        let res_f = self.builder.build_signed_int_to_float(res_i, f64_ty, "tof64").unwrap();
        self.store(dest, self.nan.box_number(&self.builder, res_f));
        Ok(())
    }

    /// Emit unsigned right shift (>>>): f64 → i32, lshr, u32 → f64, re-box.
    pub(super) fn emit_bitwise_u32(
        &mut self,
        dest: MirReg,
        l: &MirOperand,
        r: &MirOperand,
    ) -> CompileResult<()> {
        let lv = self.val(l)?;
        let rv = self.val(r)?;
        let lf = self.nan.unbox_number(&self.builder, lv);
        let rf = self.nan.unbox_number(&self.builder, rv);
        let li = self.builder.build_float_to_signed_int(lf, self.i32_ty, "ltoi32").unwrap();
        let ri = self.builder.build_float_to_signed_int(rf, self.i32_ty, "rtoi32").unwrap();
        let mask = self.i32_ty.const_int(0x1f, false);
        let shift = self.builder.build_and(ri, mask, "ushrmask").unwrap();
        let res_i = self.builder.build_right_shift(li, shift, false, "lshr").unwrap();
        // Unsigned: convert u32 to f64
        let f64_ty = self.ctx.f64_type();
        let res_f = self.builder.build_unsigned_int_to_float(res_i, f64_ty, "utof64").unwrap();
        self.store(dest, self.nan.box_number(&self.builder, res_f));
        Ok(())
    }

    pub(super) fn emit_cmp_f64(
        &mut self,
        dest: MirReg,
        l: &MirOperand,
        r: &MirOperand,
        pred: FloatPredicate,
    ) -> CompileResult<()> {
        let lv = self.val(l)?;
        let rv = self.val(r)?;
        let lf = self.nan.unbox_number(&self.builder, lv);
        let rf = self.nan.unbox_number(&self.builder, rv);
        let cmp = self.builder.build_float_compare(pred, lf, rf, "cmp").unwrap();
        self.store(dest, self.nan.box_bool(&self.builder, cmp));
        Ok(())
    }

    pub(super) fn emit_eq(
        &mut self,
        dest: MirReg,
        l: &MirOperand,
        r: &MirOperand,
    ) -> CompileResult<()> {
        let lv = self.val(l)?;
        let rv = self.val(r)?;
        let func = self.module.get_function("__bs_strict_eq").ok_or_else(|| CompileError::Codegen {
            message: "runtime helper __bs_strict_eq not found".to_string(),
        })?;
        let call = self.builder.build_call(func, &[lv.into(), rv.into()], "add_call").unwrap();
        let res = call.try_as_basic_value().basic().unwrap().into_int_value();
        self.store(dest, res);
        Ok(())
    }

    pub(super) fn emit_ne(
        &mut self,
        dest: MirReg,
        l: &MirOperand,
        r: &MirOperand,
    ) -> CompileResult<()> {
        let lv = self.val(l)?;
        let rv = self.val(r)?;
        let func = self.module.get_function("__bs_strict_ne").ok_or_else(|| CompileError::Codegen {
            message: "runtime helper __bs_strict_ne not found".to_string(),
        })?;
        let call = self.builder.build_call(func, &[lv.into(), rv.into()], "add_call").unwrap();
        let res = call.try_as_basic_value().basic().unwrap().into_int_value();
        self.store(dest, res);
        Ok(())
    }
}
