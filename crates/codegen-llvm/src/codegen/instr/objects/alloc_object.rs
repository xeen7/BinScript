#![allow(unused_imports)]
#![allow(unused_unsafe)]
use inkwell::values::BasicMetadataValueEnum;
use inkwell::IntPredicate;
use inkwell::FloatPredicate;
use mir::types::*;
use mir::BuiltinFn;
use diagnostics::{CompileError, CompileResult};

use crate::codegen::LlvmCodegen;

impl<'ctx> LlvmCodegen<'ctx> {
    #[allow(unreachable_code)]
    #[allow(unused_variables)]
    pub(in crate::codegen::instr) fn emit_instr_alloc_object(&mut self, instr: &MirInstr) -> CompileResult<()> {
        match instr {
            MirInstr::Alloc(dest, class_name) | MirInstr::AllocShared(dest, class_name) => {
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
                let (size_in_bytes, vtable_ptr) = if class_name == "Object" {
                    let vtable_g = self.module.get_global("OBJECT_VTABLE").unwrap_or_else(|| {
                        self.module.add_global(self.i64_ty, None, "OBJECT_VTABLE")
                    });
                    (16, vtable_g.as_pointer_value())
                } else {
                    let fields_count = self.get_all_fields_count(class_name);
                    let size_in_bytes = 8 * (2 + fields_count);
                    let vtable_g = self.vtables.get(class_name).ok_or_else(|| {
                        CompileError::Codegen {
                            message: format!("Vtable not found for class {}", class_name),
                        }
                    })?;
                    (size_in_bytes, vtable_g.as_pointer_value())
                };
                let alloc_fn = self.funcs["__bs_alloc"];
                let size_val = self.i64_ty.const_int(size_in_bytes as u64, false);
                let obj_val = self.builder.build_call(alloc_fn, &[vtable_ptr.into(), size_val.into()], "alloc").unwrap()
                    .try_as_basic_value()
                    .basic()
                    .unwrap()
                    .into_int_value();
                self.store(*dest, obj_val);
            }
            MirInstr::AllocAcyclic(dest, class_name) | MirInstr::AllocSharedAcyclic(dest, class_name) => {
                let (size_in_bytes, vtable_ptr) = if class_name == "Object" {
                    let vtable_g = self.module.get_global("OBJECT_VTABLE").unwrap_or_else(|| {
                        self.module.add_global(self.i64_ty, None, "OBJECT_VTABLE")
                    });
                    (16, vtable_g.as_pointer_value())
                } else {
                    let fields_count = self.get_all_fields_count(class_name);
                    let size_in_bytes = 8 * (2 + fields_count);
                    let vtable_g = self.vtables.get(class_name).ok_or_else(|| {
                        CompileError::Codegen {
                            message: format!("Vtable not found for class {}", class_name),
                        }
                    })?;
                    (size_in_bytes, vtable_g.as_pointer_value())
                };
                let alloc_fn = self.funcs["__bs_alloc_acyclic"];
                let size_val = self.i64_ty.const_int(size_in_bytes as u64, false);
                let obj_val = self.builder.build_call(alloc_fn, &[vtable_ptr.into(), size_val.into()], "alloc_acyclic").unwrap()
                    .try_as_basic_value()
                    .basic()
                    .unwrap()
                    .into_int_value();
                self.store(*dest, obj_val);
            }
            MirInstr::AllocOwned(dest, class_name) => {
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
                let (size_in_bytes, vtable_ptr) = if class_name == "Object" {
                    let vtable_g = self.module.get_global("OBJECT_VTABLE").unwrap_or_else(|| {
                        self.module.add_global(self.i64_ty, None, "OBJECT_VTABLE")
                    });
                    (16, vtable_g.as_pointer_value())
                } else {
                    let fields_count = self.get_all_fields_count(class_name);
                    let size_in_bytes = 8 * (2 + fields_count);
                    let vtable_g = self.vtables.get(class_name).ok_or_else(|| {
                        CompileError::Codegen {
                            message: format!("Vtable not found for class {}", class_name),
                        }
                    })?;
                    (size_in_bytes, vtable_g.as_pointer_value())
                };
                let alloc_fn = self.funcs["__bs_alloc_owned"];
                let size_val = self.i64_ty.const_int(size_in_bytes as u64, false);
                let obj_val = self.builder.build_call(alloc_fn, &[vtable_ptr.into(), size_val.into()], "alloc_owned").unwrap()
                    .try_as_basic_value()
                    .basic()
                    .unwrap()
                    .into_int_value();
                self.store(*dest, obj_val);
            }
            MirInstr::AllocStack(dest, class_name) => {
                let (size_in_bytes, vtable_ptr) = if class_name == "Object" {
                    let vtable_g = self.module.get_global("OBJECT_VTABLE").unwrap_or_else(|| {
                        self.module.add_global(self.i64_ty, None, "OBJECT_VTABLE")
                    });
                    (16, vtable_g.as_pointer_value())
                } else {
                    let fields_count = self.get_all_fields_count(class_name);
                    let size_in_bytes = 8 * (2 + fields_count);
                    let vtable_g = self.vtables.get(class_name).ok_or_else(|| {
                        CompileError::Codegen {
                            message: format!("Vtable not found for class {}", class_name),
                        }
                    })?;
                    (size_in_bytes, vtable_g.as_pointer_value())
                };
                
                // Allocate on the stack (use i64 array to guarantee 8-byte alignment)
                let size_i64_val = self.i64_ty.const_int((size_in_bytes / 8) as u64, false);
                let alloca_ptr_i64 = self.builder.build_array_alloca(self.i64_ty, size_i64_val, "stack_alloc_i64").unwrap();
                let alloca_ptr = self.builder.build_pointer_cast(alloca_ptr_i64, self.ptr_ty, "stack_alloc").unwrap();
                let size_val = self.i64_ty.const_int(size_in_bytes as u64, false);
                
                // zero-initialize
                let memset_fn = self.module.get_function("llvm.memset.p0.i64").unwrap_or_else(|| {
                    // Declare llvm.memset if it doesn't exist. signature: void @llvm.memset.p0.i64(ptr nocapture writeonly, i8, i64, i1 immarg)
                    let ft = self.void_ty.fn_type(&[self.ptr_ty.into(), self.i8_ty.into(), self.i64_ty.into(), self.ctx.bool_type().into()], false);
                    self.module.add_function("llvm.memset.p0.i64", ft, None)
                });
                self.builder.build_call(memset_fn, &[
                    alloca_ptr.into(),
                    self.i8_ty.const_int(0, false).into(),
                    size_val.into(),
                    self.ctx.bool_type().const_zero().into() // is_volatile = false
                ], "memset").unwrap();
                
                // Store vtable ptr
                let vtable_slot = self.builder.build_pointer_cast(alloca_ptr, self.ptr_ty, "vtable_slot").unwrap();
                self.builder.build_store(vtable_slot, vtable_ptr).unwrap();
                
                // Return NaN-boxed Object pointer
                let obj_ptr_i64 = self.builder.build_ptr_to_int(alloca_ptr, self.i64_ty, "obj_ptr_i64").unwrap();
                let tag = self.i64_ty.const_int(0xFFFE_0000_0000_0000, false);
                let boxed_ptr = self.builder.build_or(obj_ptr_i64, tag, "boxed_ptr").unwrap();
                
                self.store(*dest, boxed_ptr);
            }
            MirInstr::AllocArena(dest, class_name, region_id) => {
                if class_name == "Array" {
                    let arena_ptr = *self.arena_ptrs.get(region_id).unwrap();
                    let alloc_fn = self.funcs["__bs_alloc_arena_array"];
                    let obj_val = self.builder.build_call(alloc_fn, &[arena_ptr.into()], "alloc_arena_array").unwrap()
                        .try_as_basic_value()
                        .basic()
                        .unwrap()
                        .into_int_value();
                    self.store(*dest, obj_val);
                    return Ok(());
                }
                let (size_in_bytes, vtable_ptr) = if class_name == "Object" {
                    let vtable_g = self.module.get_global("OBJECT_VTABLE").unwrap_or_else(|| {
                        self.module.add_global(self.i64_ty, None, "OBJECT_VTABLE")
                    });
                    (16, vtable_g.as_pointer_value())
                } else {
                    let fields_count = self.get_all_fields_count(class_name);
                    let size_in_bytes = 8 * (2 + fields_count);
                    let vtable_g = self.vtables.get(class_name).ok_or_else(|| {
                        CompileError::Codegen {
                            message: format!("Vtable not found for class {}", class_name),
                        }
                    })?;
                    (size_in_bytes, vtable_g.as_pointer_value())
                };
                
                let arena_alloc_fn = self.funcs["arena_alloc"];
                let size_val = self.i64_ty.const_int(size_in_bytes as u64, false);
                let align_val = self.i64_ty.const_int(8, false);
                
                let arena_ptr = *self.arena_ptrs.get(region_id).ok_or_else(|| {
                    CompileError::Codegen {
                        message: format!("Arena pointer not found for region {}", region_id),
                    }
                })?;
                
                let raw_ptr = self.builder.build_call(arena_alloc_fn, &[arena_ptr.into(), size_val.into(), align_val.into()], "arena_alloc_call").unwrap()
                    .try_as_basic_value()
                    .basic()
                    .unwrap()
                    .into_pointer_value();
                    
                // zero-initialize
                let memset_fn = self.module.get_function("llvm.memset.p0.i64").unwrap_or_else(|| {
                    let ft = self.void_ty.fn_type(&[self.ptr_ty.into(), self.i8_ty.into(), self.i64_ty.into(), self.ctx.bool_type().into()], false);
                    self.module.add_function("llvm.memset.p0.i64", ft, None)
                });
                self.builder.build_call(memset_fn, &[
                    raw_ptr.into(),
                    self.i8_ty.const_int(0, false).into(),
                    size_val.into(),
                    self.ctx.bool_type().const_zero().into() // is_volatile = false
                ], "memset").unwrap();
                
                // Store vtable ptr
                self.builder.build_store(raw_ptr, vtable_ptr).unwrap();
                
                // If it has a drop_fn, register it in the arena's dtor_list
                if let Some(drop_fn_func) = self.drop_fns.get(class_name) {
                    let register_fn = self.funcs["arena_register_dtor"];
                    let drop_fn_ptr = drop_fn_func.as_global_value().as_pointer_value();
                    self.builder.build_call(register_fn, &[arena_ptr.into(), raw_ptr.into(), drop_fn_ptr.into()], "call_register_dtor").unwrap();
                }
                
                // Return NaN-boxed Object pointer
                let obj_ptr_i64 = self.builder.build_ptr_to_int(raw_ptr, self.i64_ty, "obj_ptr_i64").unwrap();
                let tag = self.i64_ty.const_int(0xFFFE_0000_0000_0000, false);
                let boxed_ptr = self.builder.build_or(obj_ptr_i64, tag, "boxed_ptr").unwrap();
                
                self.store(*dest, boxed_ptr);
            }
            _ => unreachable!()
        }
        Ok(())
    }
}
