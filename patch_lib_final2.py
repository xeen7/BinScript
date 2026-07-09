with open("crates/ownership-inference/src/lib.rs", "r") as f:
    code = f.read()
code = code.replace("let mut escape_sigs = std::collections::HashMap::new();", "")
with open("crates/ownership-inference/src/lib.rs", "w") as f:
    f.write(code)

with open("crates/ownership-inference/src/liveness.rs", "r") as f:
    code = f.read()

import re
code = re.sub(r"(MirInstr::Borrow\(d, s\) \| MirInstr::BorrowMut\(d, s\) => \{ defs.push\(\*d\); uses.push\(\*s\); \},)", r"\1\n        MirInstr::ForceOwnedTag(d) => { defs.push(*d); uses.push(*d); },", code)

with open("crates/ownership-inference/src/liveness.rs", "w") as f:
    f.write(code)

