import json

with open("/home/samon/.gemini/antigravity-ide/brain/d858f18d-b78c-4f4d-83cf-44932774635e/.system_generated/logs/transcript_full.jsonl", "r") as f:
    for line in f:
        data = json.loads(line)
        if "content" in data and "commit 256cd1a1204f441051b986f358940a9960fc9adb" in data["content"]:
            print(data["content"])
