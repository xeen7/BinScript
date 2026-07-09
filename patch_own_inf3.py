with open("crates/ownership-inference/src/lib.rs", "r") as f:
    code = f.read()

import re

# Find the loop over instrs
match = re.search(r"for instr in &mut block\.instrs \{(.+?)\n            \}", code, re.DOTALL)
if match:
    print("Found match!")

# Let's just use sed or Python to replace it based on lines.
