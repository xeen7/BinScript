import json
import re

with open("/home/samon/.gemini/antigravity-ide/brain/d858f18d-b78c-4f4d-83cf-44932774635e/.system_generated/logs/transcript_full.jsonl", "r") as f:
    for line in f:
        data = json.loads(line)
        if "content" in data and "2026-07-08T20:50:31Z" in data["content"] and "File Path" in data["content"]:
            # This is the view_file output we want
            content = data["content"]
            lines = content.split('\n')
            output = []
            parsing = False
            for c_line in lines:
                if "The following code has been modified" in c_line:
                    parsing = True
                    continue
                if "The above content does NOT show" in c_line or "The following is a <SYSTEM_MESSAGE>" in c_line:
                    break
                
                if parsing:
                    match = re.match(r'^\d+: (.*)$', c_line)
                    if match:
                        output.append(match.group(1))
                    elif re.match(r'^\d+:$', c_line.strip()):
                        output.append("")
            
            with open("crates/ownership-inference/src/escape.rs", "w") as out_f:
                out_f.write("\n".join(output) + "\n")
            print(f"Extracted {len(output)} lines")
            break
