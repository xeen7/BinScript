with open("crates/ownership-inference/src/lib.rs", "r") as f:
    code = f.read()

code = code.replace("let ea = escape::run_escape_analysis(func, &escape_sigs);", "let ea = escape::run_escape_analysis(func);")

with open("crates/ownership-inference/src/lib.rs", "w") as f:
    f.write(code)

with open("crates/ownership-inference/src/liveness.rs", "r") as f:
    code = f.read()

# Make sure we add ForceOwnedTag in the right place
code = code.replace("""        MirInstr::RcDecDeferred(s) => { uses.push(*s); },
        MirInstr::ForceOwnedTag(d) => { defs.push(*d); uses.push(*d); },""", """        MirInstr::RcDecDeferred(s) => { uses.push(*s); },""")

# Add it at the end of the match block, just before `}`
import re
match = re.search(r"(        MirInstr::Borrow\(d, s\) \| MirInstr::BorrowMut\(d, s\) => \{ defs.push\(\*d\); uses.push\(\*s\); \},\n    \})", code)
if match:
    code = code.replace(match.group(1), """        MirInstr::Borrow(d, s) | MirInstr::BorrowMut(d, s) => { defs.push(*d); uses.push(*s); },
        MirInstr::ForceOwnedTag(d) => { defs.push(*d); uses.push(*d); },
    }""")

with open("crates/ownership-inference/src/liveness.rs", "w") as f:
    f.write(code)
