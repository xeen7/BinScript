import re

with open("crates/codegen-llvm/src/codegen/instr/call.rs", "r") as f:
    code = f.read()

# I need to see how CallDirect is implemented
