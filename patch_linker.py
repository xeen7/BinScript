import re
content = open("crates/codegen-llvm/src/linker.rs").read()
content = content.replace("cmd.arg(\"-fsanitize=address\");", "cmd.arg(\"-fsanitize=address\");\n    cmd.arg(\"-lstdc++\");")
open("crates/codegen-llvm/src/linker.rs", "w").write(content)
