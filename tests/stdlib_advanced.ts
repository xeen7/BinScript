// ============================================================================
// BinScript Standard Library & Built-in Objects Advanced Test Suite (ES2022)
// ============================================================================

console.log("=== STARTING STDLIB ADVANCED TEST ===\n");

// ----------------------------------------------------------------------------
// 1. Primitive Wrappers & Object Mechanics
// ----------------------------------------------------------------------------
console.log("--- 1. Object Mechanics ---");
const litObj = { a: 10, b: "hello" };
console.log(litObj.a); // 10
console.log(litObj.b); // "hello"

const newObj = new Object();
newObj.nested = litObj;
newObj.status = true;
console.log(newObj.status); // true
console.log(newObj.nested.b); // "hello"

// Verify console.log of plain objects (no segfault)
console.log(litObj); // Object {}
console.log(newObj); // Object {}


// ----------------------------------------------------------------------------
// 2. Math & Number Operations
// ----------------------------------------------------------------------------
console.log("\n--- 2. Math & Numbers ---");
console.log(Math.PI > 3.14 && Math.PI < 3.15); // true
console.log(Math.E > 2.71 && Math.E < 2.72);   // true

console.log(Math.floor(4.9)); // 4
console.log(Math.ceil(4.1));  // 5
console.log(Math.round(4.5)); // 5
console.log(Math.round(4.4)); // 4
console.log(Math.abs(-123));  // 123
console.log(Math.sqrt(81));   // 9
console.log(Math.pow(2, 10)); // 1024
console.log(Math.trunc(15.95)); // 15
console.log(Math.min(50, -5)); // -5
console.log(Math.max(50, -5)); // 50

// Trigonometry
console.log(Math.sin(0)); // 0
console.log(Math.cos(0)); // 1

// Random range check
const rand = Math.random();
console.log(rand >= 0 && rand < 1); // true

// Global parsing & validation
console.log(parseInt("512")); // 512
console.log(parseInt("1111", 2)); // 15
console.log(parseInt("ff", 16)); // 255
console.log(parseFloat("12.34")); // 12.34

console.log(isNaN(NaN)); // true
console.log(isNaN(99));  // false
console.log(isNaN("not-a-number")); // true

console.log(isFinite(1000)); // true
console.log(isFinite(Infinity)); // false

console.log(Number.isInteger(123)); // true
console.log(Number.isInteger(12.3)); // false


// ----------------------------------------------------------------------------
// 3. String Methods
// ----------------------------------------------------------------------------
console.log("\n--- 3. String Mechanics ---");
const rawStr = "   BinScript compiler   ";
console.log(rawStr.length); // 24

const trimmed = rawStr.trim();
console.log(trimmed); // "BinScript compiler"
console.log(trimmed.toUpperCase()); // "BINSCRIPT COMPILER"
console.log(trimmed.charAt(0)); // "B"
console.log(trimmed.charCodeAt(0)); // 66

const parts = trimmed.split(" ");
console.log(parts.length); // 2
console.log(parts[0]); // "BinScript"
console.log(parts[1]); // "compiler"


// ----------------------------------------------------------------------------
// 4. Indexed Collections (Arrays)
// ----------------------------------------------------------------------------
console.log("\n--- 4. Array Mechanics ---");
const litArr = [10, 20, 30];
console.log(litArr.length); // 3
console.log(litArr[1]); // 20

const newArr = new Array();
newArr.push(100);
newArr.push(200);
newArr.push(300);
console.log(newArr.length); // 3
console.log(newArr[0]); // 100
console.log(newArr.pop()); // 300
console.log(newArr.length); // 2

// Array Higher-Order Functions
const numbers = [1, 2, 3, 4, 5];
const squares = numbers.map((n: number) => n * n);
console.log(squares[0]); // 1
console.log(squares[1]); // 4
console.log(squares[4]); // 25

const evens = numbers.filter((n: number) => n % 2 === 0);
console.log(evens.length); // 2
console.log(evens[0]); // 2
console.log(evens[1]); // 4

const sum = numbers.reduce((acc: number, val: number) => acc + val, 0);
console.log(sum); // 15

console.log(numbers.indexOf(3)); // 2
console.log(numbers.indexOf(10)); // -1
console.log(numbers.includes(4)); // true
console.log(numbers.includes(9)); // false


// ----------------------------------------------------------------------------
// 5. Advanced Exception Handling & Error Types
// ----------------------------------------------------------------------------
console.log("\n--- 5. Standard Errors & Exceptions ---");

function verifyError(err: any, expectedName: string, expectedMsg: string) {
    console.log(err.name);    // expectedName
    console.log(err.message); // expectedMsg
    err.customCode = 999;
    console.log(err.customCode); // 999
}

// Test instantiation of all 5 error types
const errBase = new Error("base issue");
verifyError(errBase, "Error", "base issue");

const errType = new TypeError("type mismatch");
verifyError(errType, "TypeError", "type mismatch");

const errRange = new RangeError("out of bounds");
verifyError(errRange, "RangeError", "out of bounds");

const errRef = new ReferenceError("unknown variable");
verifyError(errRef, "ReferenceError", "unknown variable");

const errSyntax = new SyntaxError("invalid code syntax");
verifyError(errSyntax, "SyntaxError", "invalid code syntax");

// Catching and re-throwing standard errors in nested contexts
function crashFunction(code: number) {
    if (code === 1) {
        throw new TypeError("Crashing with TypeError");
    } else if (code === 2) {
        throw new RangeError("Crashing with RangeError");
    } else {
        throw new Error("Crashing with Error");
    }
}

const codes = [1, 2, 3];
for (let i = 0; i < 3; i = i + 1) {
    const c = codes[i];
    try {
        crashFunction(c);
    } catch (e) {
        console.log("Caught standard error:");
        console.log(e.name);
        console.log(e.message);
    }
}

console.log("\n=== ALL ADVANCED TESTS COMPLETED SUCCESSFULLY ===");
