with open("crates/mir/src/types.rs", "r") as f:
    code = f.read()

code = code.replace("""    Move(MirReg, MirOperand),""", """    Move(MirReg, MirOperand),
    ForceOwnedTag(MirReg),""")

with open("crates/mir/src/types.rs", "w") as f:
    f.write(code)

with open("crates/codegen-llvm/src/codegen/func.rs", "r") as f:
    code = f.read()

code = code.replace("""                MirInstr::RcInc(r)
                | MirInstr::RcDec(r)
                | MirInstr::RcIncDeferred(r)
                | MirInstr::RcDecDeferred(r) => self.emit_instr_circ(instr)?,
""", """                MirInstr::RcInc(r)
                | MirInstr::RcDec(r)
                | MirInstr::RcIncDeferred(r)
                | MirInstr::RcDecDeferred(r)
                | MirInstr::ForceOwnedTag(r) => self.emit_instr_circ(instr)?,
""")

with open("crates/codegen-llvm/src/codegen/func.rs", "w") as f:
    f.write(code)

with open("crates/codegen-llvm/src/codegen/instr/memory/circ.rs", "r") as f:
    code = f.read()

impl = """            MirInstr::ForceOwnedTag(r) => {
                let val = self.load(*r);
                let mask = self.i64_ty.const_int(0x7FFF_FFFF_FFFF_FFFF, false);
                let new_val = self.builder.build_and(val, mask, "force_owned_tag").unwrap();
                self.store(*r, new_val);
            }
"""

code = code.replace("""            MirInstr::FlushRcDelta => {""", impl + """            MirInstr::FlushRcDelta => {""")

with open("crates/codegen-llvm/src/codegen/instr/memory/circ.rs", "w") as f:
    f.write(code)

