with open("crates/ownership-inference/src/lib.rs", "r") as f:
    code = f.read()

import re

# We will match `MirInstr::AllocShared(.*?)\n                    \}` and add `classify::MemoryClass::Borrow => unreachable!(),`
code = re.sub(r"(MirInstr::AllocShared.*?\n                    \})", r"\1\n                    classify::MemoryClass::Borrow => unreachable!(),", code)

# Let's also do AllocSharedClosure
code = re.sub(r"(MirInstr::AllocSharedClosure.*?\n                        \})", r"\1\n                        classify::MemoryClass::Borrow => unreachable!(),", code)

# AllocSharedGenerator
code = re.sub(r"(MirInstr::AllocSharedGenerator.*?\n                            \})", r"\1\n                            classify::MemoryClass::Borrow => unreachable!(),", code)

with open("crates/ownership-inference/src/lib.rs", "w") as f:
    f.write(code)

