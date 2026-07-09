with open("crates/ownership-inference/src/lib.rs", "r") as f:
    code = f.read()

patch = """                    classify::MemoryClass::Shared => {
                        *instr = MirInstr::AllocShared(*dest, class_name.clone());
                    }
                }
            } else if let MirInstr::CallDirect(dest, target, args) = instr {
                let mem_class = classes.get_class(*dest);
                if mem_class == classify::MemoryClass::Owned {
                    if target == "__bs_string_concat" {
                        *instr = MirInstr::CallDirect(*dest, "__bs_string_concat_owned".to_string(), args.clone());
                    } else if target == "__bs_number_to_string" {
                        *instr = MirInstr::CallDirect(*dest, "__bs_number_to_string_owned".to_string(), args.clone());
                    } else if target == "__bs_boolean_to_string" {
                        *instr = MirInstr::CallDirect(*dest, "__bs_boolean_to_string_owned".to_string(), args.clone());
                    }
                }
            }"""

code = code.replace("""                    classify::MemoryClass::Shared => {
                        *instr = MirInstr::AllocShared(*dest, class_name.clone());
                    }
                }
            }""", patch)

with open("crates/ownership-inference/src/lib.rs", "w") as f:
    f.write(code)
