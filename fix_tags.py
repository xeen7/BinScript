import os
import re

for root, _, files in os.walk("rt-stubs/src"):
    for file in files:
        if not file.endswith(".rs"): continue
        path = os.path.join(root, file)
        
        # skip array because it should NOT drop 0xFFFE
        if path == "rt-stubs/src/array/mod.rs": continue
        
        with open(path, "r") as f:
            content = f.read()
            
        new_content = content.replace("tag == 0xFFFC_0000_0000_0000", "(tag == 0xFFFC_0000_0000_0000 || tag == 0xFFFE_0000_0000_0000)")
        new_content = new_content.replace("tag != 0xFFFC_0000_0000_0000", "tag != 0xFFFC_0000_0000_0000 && tag != 0xFFFE_0000_0000_0000")
        
        new_content = new_content.replace("target_tag == 0xFFFC_0000_0000_0000", "(target_tag == 0xFFFC_0000_0000_0000 || target_tag == 0xFFFE_0000_0000_0000)")
        new_content = new_content.replace("target_tag != 0xFFFC_0000_0000_0000", "target_tag != 0xFFFC_0000_0000_0000 && target_tag != 0xFFFE_0000_0000_0000")
        
        new_content = new_content.replace("source_tag == 0xFFFC_0000_0000_0000", "(source_tag == 0xFFFC_0000_0000_0000 || source_tag == 0xFFFE_0000_0000_0000)")
        new_content = new_content.replace("source_tag != 0xFFFC_0000_0000_0000", "source_tag != 0xFFFC_0000_0000_0000 && source_tag != 0xFFFE_0000_0000_0000")

        new_content = new_content.replace("proto_tag == 0xFFFC_0000_0000_0000", "(proto_tag == 0xFFFC_0000_0000_0000 || proto_tag == 0xFFFE_0000_0000_0000)")
        new_content = new_content.replace("proto_tag != 0xFFFC_0000_0000_0000", "proto_tag != 0xFFFC_0000_0000_0000 && proto_tag != 0xFFFE_0000_0000_0000")
        
        with open(path, "w") as f:
            f.write(new_content)

print("Done fixing tags!")
