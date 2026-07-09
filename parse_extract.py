import re

with open("extract.out", "r") as f:
    lines = f.readlines()

output_lines = []
start_parsing = False
for line in lines:
    if "The following code has been modified" in line:
        start_parsing = True
        continue
    if "The above content does NOT show" in line:
        break
        
    if start_parsing:
        match = re.match(r'^\d+: (.*)$', line)
        if match:
            output_lines.append(match.group(1))
        elif re.match(r'^\d+:$', line.strip()):
            output_lines.append("")

with open("crates/ownership-inference/src/escape.rs", "w") as f:
    f.write("\n".join(output_lines) + "\n")
