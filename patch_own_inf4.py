with open("crates/ownership-inference/src/lib.rs", "r") as f:
    code = f.read()

import re

target = r"            for instr in &mut block\.instrs \{(.*?)\n            \}"

def replacer(match):
    inner = match.group(1)
    
    # We replace the loop with:
    res = "            let mut new_instrs = Vec::new();\n"
    res += "            for instr in block.instrs.drain(..) {\n"
    res += "                let mut instr = instr;\n"
    res += "                let mut inject_force_owned = None;\n"
    
    # Replace the body by replacing references to `*instr` with `instr`? No.
    # The body matches `if let MirInstr::Alloc(dest, class_name) = instr {`
    # We change it to `if let MirInstr::Alloc(dest, class_name) = &instr {`
    inner = inner.replace("if let MirInstr::Alloc(dest, class_name) = instr {", "if let MirInstr::Alloc(dest, class_name) = &instr {")
    inner = inner.replace("*instr =", "instr =")
    
    # Add the CallDirect handler before the end of the loop body
    extra = """
                } else if let MirInstr::CallDirect(dest, target, _) | MirInstr::CallPure(dest, target, _) | MirInstr::CallBuiltin(dest, target, _) = &instr {
                    let mem_class = classes.get_class(*dest);
                    if mem_class == classify::MemoryClass::Owned {
                        let sig_opt = crate::native_sigs::NativeSignature::get(&target.to_string());
                        if sig_opt.map_or(false, |s| s.returns_fresh_allocation) {
                            inject_force_owned = Some(*dest);
                        }
                    }
"""
    
    # Where does `extra` go? After the if let Alloc
    # Actually, the original inner has:
    #                 if let MirInstr::Alloc(dest, class_name) = instr {
    #                     ...
    #                 }
    # So we just append it before we push the instr.
    
    res += inner
    res += extra
    res += "                new_instrs.push(instr);\n"
    res += "                if let Some(dest) = inject_force_owned {\n"
    res += "                    new_instrs.push(MirInstr::ForceOwnedTag(dest));\n"
    res += "                }\n"
    res += "            }\n"
    res += "            block.instrs = new_instrs;"
    return res

code = re.sub(target, replacer, code, flags=re.DOTALL)

with open("crates/ownership-inference/src/lib.rs", "w") as f:
    f.write(code)
