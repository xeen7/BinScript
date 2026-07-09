import json
with open("/home/samon/.gemini/antigravity-ide/brain/d858f18d-b78c-4f4d-83cf-44932774635e/.system_generated/logs/transcript_full.jsonl", "r") as f:
    for line in f:
        data = json.loads(line)
        if "tool_calls" in data:
            for call in data["tool_calls"]:
                if call["name"] in ["replace_file_content", "multi_replace_file_content", "write_to_file"]:
                    args = call["args"]
                    target = args.get("TargetFile", args.get("targetFile", ""))
                    if "escape.rs" in target:
                        content = json.dumps(args)
                        if "param_escapes" in content:
                            print(f"--- MATCH AT {data.get('created_at')} ---")
                            print(json.dumps(args, indent=2))
