import json

with open("crates/ownership-inference/src/escape.rs", "r") as f:
    content = f.read()

with open("/home/samon/.gemini/antigravity-ide/brain/d858f18d-b78c-4f4d-83cf-44932774635e/.system_generated/logs/transcript_full.jsonl", "r") as f:
    for line in f:
        data = json.loads(line)
        if "tool_calls" in data:
            for call in data["tool_calls"]:
                if call["name"] == "replace_file_content":
                    args = call["args"]
                    if "escape.rs" in args.get("TargetFile", ""):
                        target = args["TargetContent"]
                        replacement = args["ReplacementContent"]
                        if target in content:
                            content = content.replace(target, replacement)
                        else:
                            print(f"FAILED TO MATCH: {target[:50]}")
                elif call["name"] == "multi_replace_file_content":
                    args = call["args"]
                    if "escape.rs" in args.get("TargetFile", ""):
                        for chunk in args["ReplacementChunks"]:
                            target = chunk["TargetContent"]
                            replacement = chunk["ReplacementContent"]
                            if target in content:
                                content = content.replace(target, replacement)
                            else:
                                print(f"FAILED TO MATCH CHUNK: {target[:50]}")

with open("crates/ownership-inference/src/escape.rs", "w") as f:
    f.write(content)
