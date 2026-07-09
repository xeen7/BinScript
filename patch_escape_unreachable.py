with open("crates/ownership-inference/src/escape.rs", "r") as f:
    lines = f.readlines()

with open("crates/ownership-inference/src/escape.rs", "w") as f:
    for i, line in enumerate(lines):
        if 80 <= i <= 86:
            # We skip lines 82 to 87 (0-indexed 81 to 86)
            if "MirInstr::Return" in line or "MirInstr::Throw" in line or "ea.mark_escape(*r, EscapeFact::Return);" in line:
                continue
        f.write(line)
