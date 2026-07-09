with open("crates/ownership-inference/src/lib.rs", "r") as f:
    code = f.read()

import re

# We can find all `classify::MemoryClass::Shared => { ... }` and append `classify::MemoryClass::Borrow => unreachable!(),`
# The blocks are slightly different.

code = code.replace("""                    classify::MemoryClass::Shared => {
                        *instr = MirInstr::AllocShared(*dest, class_name.clone());
                    }
                }""", """                    classify::MemoryClass::Shared => {
                        *instr = MirInstr::AllocShared(*dest, class_name.clone());
                    }
                    classify::MemoryClass::Borrow => unreachable!(),
                }""")

code = code.replace("""                        classify::MemoryClass::Shared => {
                            *instr = MirInstr::AllocSharedClosure(*dest, *func_idx, env_vars.clone());
                        }
                    }""", """                        classify::MemoryClass::Shared => {
                            *instr = MirInstr::AllocSharedClosure(*dest, *func_idx, env_vars.clone());
                        }
                        classify::MemoryClass::Borrow => unreachable!(),
                    }""")

code = code.replace("""                            classify::MemoryClass::Shared => {
                                *instr = MirInstr::AllocSharedGenerator(*dest, *func_idx, env_vars.clone());
                            }
                        }""", """                            classify::MemoryClass::Shared => {
                                *instr = MirInstr::AllocSharedGenerator(*dest, *func_idx, env_vars.clone());
                            }
                            classify::MemoryClass::Borrow => unreachable!(),
                        }""")

with open("crates/ownership-inference/src/lib.rs", "w") as f:
    f.write(code)
