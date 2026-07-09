with open("crates/ownership-inference/src/lib.rs", "r") as f:
    code = f.read()

import re

# We need to insert ForceOwnedTag after CallDirect / CallPure / CallBuiltin if returns_fresh
# Wait, it's easier to iterate through instrs and build a new instrs vector.

# Let's replace the whole `for instr in &mut block.instrs` with a while loop or build a new vector.

# Here is the target section:
target = """            for instr in &mut block.instrs {
                if let MirInstr::Alloc(dest, class_name) = instr {"""

replacement = """            let mut new_instrs = Vec::new();
            for instr in block.instrs.drain(..) {
                let mut instr = instr;
                let mut inject_force_owned = None;

                if let MirInstr::Alloc(dest, class_name) = &instr {"""

code = code.replace(target, replacement)


target2 = """                    classify::MemoryClass::Shared => {
                        instr = MirInstr::AllocShared(*dest, class_name.clone());
                    }
                }
            }"""

replacement2 = """                    classify::MemoryClass::Shared => {
                        instr = MirInstr::AllocShared(*dest, class_name.clone());
                    }
                }
            } else if let MirInstr::CallDirect(dest, target, _) | MirInstr::CallPure(dest, target, _) = &instr {
                let mem_class = classes.get_class(*dest);
                if mem_class == classify::MemoryClass::Owned {
                    let sig_opt = crate::native_sigs::NativeSignature::get(target);
                    if sig_opt.map_or(false, |s| s.returns_fresh_allocation) {
                        inject_force_owned = Some(*dest);
                    }
                }
            }

            new_instrs.push(instr);
            if let Some(dest) = inject_force_owned {
                new_instrs.push(MirInstr::ForceOwnedTag(dest));
            }
        }
        block.instrs = new_instrs;
"""

code = code.replace("""                    classify::MemoryClass::Shared => {
                        *instr = MirInstr::AllocShared(*dest, class_name.clone());
                    }
                }
            }""", replacement2)

with open("crates/ownership-inference/src/lib.rs", "w") as f:
    f.write(code)
