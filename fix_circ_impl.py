with open("crates/codegen-llvm/src/codegen/instr/memory/circ.rs", "r") as f:
    code = f.read()

# Remove the appended text
code = code.replace("""    pub(in crate::codegen::instr) fn emit_instr_force_owned_tag(&mut self, instr: &mir::MirInstr) -> CompileResult<()> {
        if let mir::MirInstr::ForceOwnedTag(reg) = instr {
            let val = self.load(*reg);
            let mask = self.i64_ty.const_int(0x7FFF_FFFF_FFFF_FFFF, false);
            let new_val = self.builder.build_and(val, mask, "force_owned_tag").unwrap();
            self.store(*reg, new_val);
        }
        Ok(())
    }
""", "")

# Add it right before the last `}`
import re
match = re.search(r"(\}\n*)$", code)
if match:
    code = code[:match.start()] + """
    pub(in crate::codegen::instr) fn emit_instr_force_owned_tag(&mut self, instr: &mir::MirInstr) -> CompileResult<()> {
        if let mir::MirInstr::ForceOwnedTag(reg) = instr {
            let val = self.load(*reg);
            let mask = self.i64_ty.const_int(0x7FFF_FFFF_FFFF_FFFF, false);
            let new_val = self.builder.build_and(val, mask, "force_owned_tag").unwrap();
            self.store(*reg, new_val);
        }
        Ok(())
    }
}
"""

with open("crates/codegen-llvm/src/codegen/instr/memory/circ.rs", "w") as f:
    f.write(code)
