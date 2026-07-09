with open("crates/ownership-inference/src/liveness.rs", "r") as f:
    code = f.read()

code = code.replace("MirInstr::RcDecDeferred(s) => { uses.push(*s); },", "MirInstr::RcDecDeferred(s) => { uses.push(*s); },\n        MirInstr::ForceOwnedTag(d) => { defs.push(*d); uses.push(*d); },")

with open("crates/ownership-inference/src/liveness.rs", "w") as f:
    f.write(code)
