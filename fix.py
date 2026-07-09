with open("crates/ownership-inference/src/lib.rs", "r") as f:
    text = f.read()

text = text.replace("""            } else {
                block.instrs = new_instrs;
            }
    if true {""", """            } else {
                block.instrs = new_instrs;
            }
        }
    }

    if true {""")

with open("crates/ownership-inference/src/lib.rs", "w") as f:
    f.write(text)
