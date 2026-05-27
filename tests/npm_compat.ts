import { join, resolve } from 'path';
import { readFileSync, writeFileSync, existsSync } from 'fs';
import { platform, arch } from 'os';

// 1. Path Join
const joined = join("dir", "file.txt");
console.log(joined); // dir/file.txt

// 2. Platform & Arch Check
const plat = platform();
console.log(plat); // linux

const architecture = arch();
console.log(architecture); // x64

// 3. File Operations
const testFile = "build/test_file.txt";

// Write content to a file
writeFileSync(testFile, "Hello from dynamic file system compatibility!");

// Verify existence
if (existsSync(testFile)) {
    console.log(42); // 42 -> indicates file was successfully written & exists
}

// Read back the content
const content = readFileSync(testFile);
console.log(content); // Hello from dynamic file system compatibility!
