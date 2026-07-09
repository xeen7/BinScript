import json
with open("/home/samon/.gemini/antigravity-ide/brain/d858f18d-b78c-4f4d-83cf-44932774635e/.system_generated/logs/transcript_full.jsonl", "r") as f:
    for line in f:
        data = json.loads(line)
        if "content" in data and "2026-07-08T20:50:31Z" in data["content"] and "File Path" in data["content"]:
            print(data["content"])
