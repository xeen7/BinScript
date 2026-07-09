import json

with open("/home/samon/.gemini/antigravity-ide/brain/d858f18d-b78c-4f4d-83cf-44932774635e/.system_generated/logs/transcript_full.jsonl", "r") as f:
    for line in f:
        data = json.loads(line)
        if "tool_calls" in data:
            for call in data["tool_calls"]:
                if call["name"] == "replace_file_content" or call["name"] == "multi_replace_file_content":
                    args = call["args"]
                    target_file = ""
                    if "TargetFile" in args:
                        target_file = args["TargetFile"]
                    elif "targetFile" in args:
                        target_file = args["targetFile"]
                    if "escape.rs" in target_file:
                        print(f"--- EDIT AT {data['created_at']} ---")
                        print(json.dumps(args, indent=2))
        if "content" in data and "2026-07-08T20:23:19Z" in data["content"] and "commit 256cd" in data["content"]:
            print(f"--- DIFF AT {data['created_at']} ---")
            print(data["content"])

