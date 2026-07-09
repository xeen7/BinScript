import re

with open("rt-stubs/src/objects/builtins/object.rs", "r") as f:
    code = f.read()

# 1. Remove is_target_object entirely:
code = code.replace("""        let is_target_object = if target_tag == 0xFFFC_0000_0000_0000 {
            let header = (target & 0x0000_FFFF_FFFF_FFFF) as *mut u8;
            let header = header.wrapping_sub(crate::circ::CircHeader::SIZE) as *const crate::circ::CircHeader;
            let flags = unsafe { (*header).flags.load(std::sync::atomic::Ordering::Relaxed) };
            (flags & crate::circ::VTABLE_PTR) != 0
        } else {
            true
        };
        if !is_target_object { return target; }""", "")

# 2. Replace `let is_object ... if is_object {` with just nothing, and remove the `}` at the end.
code = code.replace("""            let is_object = if tag == 0xFFFC_0000_0000_0000 {
                let header = obj_ptr.wrapping_sub(crate::circ::CircHeader::SIZE) as *const crate::circ::CircHeader;
                let flags = unsafe { (*header).flags.load(std::sync::atomic::Ordering::Relaxed) };
                (flags & crate::circ::VTABLE_PTR) != 0
            } else {
                true
            };
            if is_object {""", "")

code = code.replace("""            let is_source_object = if source_tag == 0xFFFC_0000_0000_0000 {
                let header = source_ptr.wrapping_sub(crate::circ::CircHeader::SIZE) as *const crate::circ::CircHeader;
                let flags = unsafe { (*header).flags.load(std::sync::atomic::Ordering::Relaxed) };
                (flags & crate::circ::VTABLE_PTR) != 0
            } else {
                true
            };
            if is_source_object {""", "")

# Now remove the extra `}` blocks:
code = code.replace("""                    }
                }
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
    array
}""")

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

    for (k, val) in props_to_copy {""")


code = code.replace("""                }
            }
            }
        }
    }
    crate::circ::circ_inc_tagged(target);""", """                }
            }
        }
    }
    crate::circ::circ_inc_tagged(target);""")

with open("rt-stubs/src/objects/builtins/object.rs", "w") as f:
    f.write(code)


with open("rt-stubs/src/objects/spread.rs", "r") as f:
    spread = f.read()

spread = spread.replace("""let target_ptr = target_payload as *mut u8;
    let is_target_object = if target_tag == 0xFFFC_0000_0000_0000 {
        let header = target_ptr.wrapping_sub(crate::circ::CircHeader::SIZE) as *const crate::circ::CircHeader;
        let flags = unsafe { (*header).flags.load(std::sync::atomic::Ordering::Relaxed) };
        (flags & crate::circ::VTABLE_PTR) != 0
    } else {
        true
    };
    if !is_target_object {
        return target_tagged;
    }""", """let target_ptr = target_payload as *mut u8;""")

spread = spread.replace("""            let src_ptr = src_payload as *mut u8;
            let is_source_object = if source_tag == 0xFFFC_0000_0000_0000 {
                let header = src_ptr.wrapping_sub(crate::circ::CircHeader::SIZE) as *const crate::circ::CircHeader;
                let flags = unsafe { (*header).flags.load(std::sync::atomic::Ordering::Relaxed) };
                (flags & crate::circ::VTABLE_PTR) != 0
            } else {
                true
            };
            if is_source_object {""", """            let src_ptr = src_payload as *mut u8;""")

spread = spread.replace("""            }
            }
        }
    } else if source_tag == 0xFFF8_0000_0000_0000 { // TAG_JSON_TAPE""", """            }
        }
    } else if source_tag == 0xFFF8_0000_0000_0000 { // TAG_JSON_TAPE""")

with open("rt-stubs/src/objects/spread.rs", "w") as f:
    f.write(spread)
