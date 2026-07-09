with open("crates/ownership-inference/src/escape.rs", "r") as f:
    content = f.read()

content = content.replace("""                EscapeFact::Store |
                EscapeFact::Capture |
                EscapeFact::UnknownCall |
                EscapeFact::StoreGlobal |
                EscapeFact::Return""", """                EscapeFact::Store |
                EscapeFact::Capture |
                EscapeFact::UnknownCall |
                EscapeFact::StoreGlobal""")

with open("crates/ownership-inference/src/escape.rs", "w") as f:
    f.write(content)
