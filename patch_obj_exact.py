import re

with open("rt-stubs/src/objects/builtins/object.rs", "r") as f:
    code = f.read()

# Replace tag checks
code = code.replace("if tag == 0xFFF6_0000_0000_0000 {", "if tag == 0xFFF6_0000_0000_0000 || tag == 0xFFFC_0000_0000_0000 || tag == 0xFFFE_0000_0000_0000 {")
code = code.replace("if source_tag == 0xFFF6_0000_0000_0000 {", "if source_tag == 0xFFF6_0000_0000_0000 || source_tag == 0xFFFC_0000_0000_0000 || source_tag == 0xFFFE_0000_0000_0000 {")

code = code.replace("if target_tag == 0xFFF6_0000_0000_0000 {", """if target_tag == 0xFFF6_0000_0000_0000 || target_tag == 0xFFFC_0000_0000_0000 || target_tag == 0xFFFE_0000_0000_0000 {
        let is_target_object = if target_tag == 0xFFFC_0000_0000_0000 {
            let header = (target & 0x0000_FFFF_FFFF_FFFF) as *mut u8;
            let header = header.wrapping_sub(crate::circ::CircHeader::SIZE) as *const crate::circ::CircHeader;
            let flags = unsafe { (*header).flags.load(std::sync::atomic::Ordering::Relaxed) };
            (flags & crate::circ::VTABLE_PTR) != 0
        } else {
            true
        };
        if !is_target_object { return target; }""")

def replace_vtable(content, search, replace):
    return content.replace(search, replace)

code = code.replace("            let vtable_ptr = *(obj_ptr as *const *const VTable);", """            let is_object = if tag == 0xFFFC_0000_0000_0000 {
                let header = obj_ptr.wrapping_sub(crate::circ::CircHeader::SIZE) as *const crate::circ::CircHeader;
                let flags = unsafe { (*header).flags.load(std::sync::atomic::Ordering::Relaxed) };
                (flags & crate::circ::VTABLE_PTR) != 0
            } else {
                true
            };
            if is_object {
                let vtable_ptr = *(obj_ptr as *const *const VTable);""")

code = code.replace("            let vtable_ptr = *(source_ptr as *const *const VTable);", """            let is_source_object = if source_tag == 0xFFFC_0000_0000_0000 {
                let header = source_ptr.wrapping_sub(crate::circ::CircHeader::SIZE) as *const crate::circ::CircHeader;
                let flags = unsafe { (*header).flags.load(std::sync::atomic::Ordering::Relaxed) };
                (flags & crate::circ::VTABLE_PTR) != 0
            } else {
                true
            };
            if is_source_object {
                let vtable_ptr = *(source_ptr as *const *const VTable);""")

# Now we need to insert the closing `}` for `if is_object` and `if is_source_object`.
# __bs_object_keys:
# 60:                 }
# 61:             }
# 62:         }
# 63:     }
code = code.replace("""                    }
                }
            }
        }
    }
    array
}""", """                    }
                }
            }
            }
        }
    }
    array
}""")

# __bs_object_rest:
code = code.replace("""                            }
                        }
                    }
                }
            }
            
            {
                let props_slot = unsafe { obj_ptr.add(8) as *mut *mut std::collections::HashMap<String, u64> };
                if !unsafe { *props_slot }.is_null() {
                    let map = unsafe { &**props_slot };
                    for (k, &val) in map.iter() {
                        if k != "[[PrimitiveValue]]" && !k.starts_with("__") {
                            if !excluded_keys.contains(k) {
                                props_to_copy.push((k.clone(), val));
                            }
                        }
                    }
                }
            }
        }
    }

    for (k, val) in props_to_copy {""", """                            }
                        }
                    }
                }
            }
            
            {
                let props_slot = unsafe { obj_ptr.add(8) as *mut *mut std::collections::HashMap<String, u64> };
                if !unsafe { *props_slot }.is_null() {
                    let map = unsafe { &**props_slot };
                    for (k, &val) in map.iter() {
                        if k != "[[PrimitiveValue]]" && !k.starts_with("__") {
                            if !excluded_keys.contains(k) {
                                props_to_copy.push((k.clone(), val));
                            }
                        }
                    }
                }
            }
            }
        }
    }

    for (k, val) in props_to_copy {""")

# __bs_object_values:
code = code.replace("""                    }
                }
            }
        }
    }
    array
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_object_entries(obj: u64) -> u64 {""", """                    }
                }
            }
            }
        }
    }
    array
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_object_entries(obj: u64) -> u64 {""")

# __bs_object_entries:
code = code.replace("""                    }
                }
            }
        }
    }
    array
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_object_assign(target: u64, source: u64) -> u64 {""", """                    }
                }
            }
            }
        }
    }
    array
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_object_assign(target: u64, source: u64) -> u64 {""")

# __bs_object_assign:
code = code.replace("""                }
            }
        }
    }
    crate::circ::circ_inc_tagged(target);""", """                }
            }
            }
        }
    }
    crate::circ::circ_inc_tagged(target);""")

with open("rt-stubs/src/objects/builtins/object.rs", "w") as f:
    f.write(code)
