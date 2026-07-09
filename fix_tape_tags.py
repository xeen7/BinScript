import re

with open("rt-stubs/src/json/tape.rs", "r") as f:
    code = f.read()

# __bs_prop_get
code = code.replace("if tag == 0xFFFB_0000_0000_0000 {", "if tag == 0xFFFB_0000_0000_0000 || tag == 0x7FFB_0000_0000_0000 || tag == 0x7FFA_0000_0000_0000 {")
code = code.replace("if tag == 0xFFF7_0000_0000_0000 {", "if tag == 0xFFF7_0000_0000_0000 || tag == 0x7FF7_0000_0000_0000 {")
code = code.replace("if tag != 0xFFF6_0000_0000_0000 && tag != 0xFFFC_0000_0000_0000 && tag != 0xFFFE_0000_0000_0000 {", "if tag != 0xFFF6_0000_0000_0000 && tag != 0xFFFC_0000_0000_0000 && tag != 0xFFFE_0000_0000_0000 && tag != 0x7FF6_0000_0000_0000 {")

with open("rt-stubs/src/json/tape.rs", "w") as f:
    f.write(code)
