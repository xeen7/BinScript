

function assertEqual(actual: any, expected: any, message: string) {
    if (actual !== expected) {
        throw new Error("Assert failed: " + message + " | expected " + expected + ", got " + actual);
    }
}

function runGlobalFunctionsTests() {
    console.log("--- Global Functions ---");
    
    assertEqual(parseInt("123"), 123, "parseInt basic");
    assertEqual(parseInt("101", 2), 5, "parseInt radix 2");
    assertEqual(parseInt("  -45abc"), -45, "parseInt signed/trailing");
    assertEqual(isNaN(parseInt("abc")), true, "parseInt NaN");

    assertEqual(parseFloat("3.14159"), 3.14159, "parseFloat basic");
    assertEqual(parseFloat("  -0.5abc"), -0.5, "parseFloat signed/trailing");
    assertEqual(isNaN(parseFloat("xyz")), true, "parseFloat NaN");

    assertEqual(isNaN(NaN), true, "isNaN(NaN)");
    assertEqual(isNaN(42), false, "isNaN(42)");
    assertEqual(isFinite(42), true, "isFinite(42)");
    assertEqual(isFinite(NaN), false, "isFinite(NaN)");

    assertEqual(encodeURIComponent("hello world!"), "hello%20world!", "encodeURIComponent");
    assertEqual(decodeURIComponent("hello%20world!"), "hello world!", "decodeURIComponent");
}

function runDateTests() {
    console.log("--- Date Object ---");
    
    const now = Date.now();
    assertEqual(typeof now === "number" && now > 0, true, "Date.now()");

    const date = new Date(1715990400000); 
    assertEqual(date.getTime(), 1715990400000, "Date getTime");
    assertEqual(date.getFullYear(), 2024, "Date getFullYear");
    assertEqual(date.getMonth(), 4, "Date getMonth (0-indexed)");
    assertEqual(date.getDate(), 18, "Date getDate");
}

function runMapTests() {
    console.log("--- Map Object ---");
    const map = new Map();
    assertEqual(map.size, 0, "Map size 0");

    map.set("a", 1);
    map.set("b", 2);
    assertEqual(map.size, 2, "Map size 2");
    assertEqual(map.get("a"), 1, "Map get a");
    assertEqual(map.has("b"), true, "Map has b");
    assertEqual(map.has("c"), false, "Map does not have c");

    map.delete("a");
    assertEqual(map.size, 1, "Map size after delete");
    assertEqual(map.has("a"), false, "Map deleted has");

    map.clear();
    assertEqual(map.size, 0, "Map size after clear");
}

function runSetTests() {
    console.log("--- Set Object ---");
    const set = new Set();
    assertEqual(set.size, 0, "Set size 0");

    set.add("apple");
    set.add("banana");
    set.add("apple"); 
    assertEqual(set.size, 2, "Set size 2 (no duplicates)");
    assertEqual(set.has("banana"), true, "Set has banana");

    set.delete("apple");
    assertEqual(set.size, 1, "Set size after delete");
    assertEqual(set.has("apple"), false, "Set deleted has");

    set.clear();
    assertEqual(set.size, 0, "Set size after clear");
}

function runJSONTests() {
    console.log("--- JSON Object ---");
    const obj = { name: "Antigravity", age: 1.0, active: true };
    const jsonStr = JSON.stringify(obj);
    
    assertEqual(jsonStr, '{"name":"Antigravity","age":1,"active":true}', "JSON stringify");

    const parsed = JSON.parse(jsonStr);
    assertEqual(parsed.name, "Antigravity", "JSON parse name");
    assertEqual(parsed.age, 1.0, "JSON parse age");
    assertEqual(parsed.active, true, "JSON parse active");
}

function runRegExpTests() {
    console.log("--- RegExp Object ---");
    const regex = new RegExp("^hello (\\w+)", "i");
    assertEqual(regex.test("hello world"), true, "RegExp test positive");
    assertEqual(regex.test("HELLO universe"), true, "RegExp test case-insensitive");
    assertEqual(regex.test("goodbye world"), false, "RegExp test negative");

    const execRes = regex.exec("HELLO beautiful");
    assertEqual(execRes !== null, true, "RegExp exec not null");
    assertEqual(execRes![0], "HELLO beautiful", "RegExp exec match");
    assertEqual(execRes![1], "beautiful", "RegExp exec capture group");
    assertEqual(execRes!.index, 0, "RegExp exec index");
}

function runObjectTests() {
    console.log("--- Object Object ---");
    const empty = new Object();
    assertEqual(Object.keys(empty).length, 0, "new Object keys length");

    const obj = { x: 10, y: "hello" };
    const keys = Object.keys(obj);
    assertEqual(keys.length, 2, "Object.keys length");
    assertEqual(keys[0], "x", "Object.keys[0]");
    assertEqual(keys[1], "y", "Object.keys[1]");

    const values = Object.values(obj);
    assertEqual(values.length, 2, "Object.values length");
    assertEqual(values[0], 10, "Object.values[0]");
    assertEqual(values[1], "hello", "Object.values[1]");

    const entries = Object.entries(obj);
    assertEqual(entries.length, 2, "Object.entries length");
    assertEqual(entries[0][0], "x", "Object.entries[0][0]");
    assertEqual(entries[0][1], 10, "Object.entries[0][1]");
    assertEqual(entries[1][0], "y", "Object.entries[1][0]");
    assertEqual(entries[1][1], "hello", "Object.entries[1][1]");

    const target = { a: 1 };
    const source = { b: 2, c: 3 };
    const assigned = Object.assign(target, source);
    assertEqual(assigned.a, 1, "Object.assign target key a");
    assertEqual(assigned.b, 2, "Object.assign target key b");
    assertEqual(assigned.c, 3, "Object.assign target key c");
    assertEqual(Object.keys(assigned).length, 3, "Object.assign target keys count");
}

function runNumberTests() {
    console.log("--- Number & Math Objects ---");
    
    assertEqual(Number.MAX_VALUE > 0, true, "Number.MAX_VALUE");
    assertEqual(Number.MIN_VALUE > 0, true, "Number.MIN_VALUE");
    assertEqual(Number.isNaN(Number.NaN), true, "Number.NaN");
    assertEqual(Number.EPSILON > 0, true, "Number.EPSILON");
    assertEqual(Math.PI > 3.14 && Math.PI < 3.15, true, "Math.PI");
    assertEqual(Math.E > 2.7 && Math.E < 2.8, true, "Math.E");

    assertEqual(Number.isInteger(42.0), true, "Number.isInteger (int)");
    assertEqual(Number.isInteger(42.42), false, "Number.isInteger (float)");
    assertEqual(Number.isSafeInteger(9007199254740991.0), true, "Number.isSafeInteger (max)");
    assertEqual(Number.isSafeInteger(9007199254740992.0), false, "Number.isSafeInteger (overflow)");
    assertEqual(Number.isNaN(42.0), false, "Number.isNaN (number)");
    assertEqual(Number.isFinite(Infinity), false, "Number.isFinite (Infinity)");
    assertEqual(Number.isFinite(42.0), true, "Number.isFinite (number)");
    assertEqual(Number.parseInt("123"), 123.0, "Number.parseInt");
    assertEqual(Number.parseFloat("12.34"), 12.34, "Number.parseFloat");

    const num = 123.456;
    assertEqual(num.toFixed(2), "123.46", "num.toFixed");
    assertEqual(num.toPrecision(4), "123.5", "num.toPrecision");
    assertEqual((255.0).toString(16), "ff", "num.toString(16)");
    assertEqual((42.0).toString(), "42", "num.toString()");
}

function runWeakMapTests() {
    console.log("--- WeakMap Object ---");
    const wm = new WeakMap();
    const key1 = { id: 1 };
    const key2 = { id: 2 };

    wm.set(key1, "value1");
    wm.set(key2, "value2");

    assertEqual(wm.has(key1), true, "WeakMap has key1");
    assertEqual(wm.has({ id: 1 }), false, "WeakMap does not have different object key reference");
    assertEqual(wm.get(key1), "value1", "WeakMap get key1");
    assertEqual(wm.get(key2), "value2", "WeakMap get key2");

    assertEqual(wm.delete(key1), true, "WeakMap delete key1");
    assertEqual(wm.has(key1), false, "WeakMap has key1 after delete");
    assertEqual(wm.delete(key1), false, "WeakMap delete key1 again");

    const key3 = { id: 3 };
    const wm2 = new WeakMap([[key3, "value3"]]);
    assertEqual(wm2.has(key3), true, "WeakMap2 initialized with key3");
    assertEqual(wm2.get(key3), "value3", "WeakMap2 get key3");
}

function runWeakSetTests() {
    console.log("--- WeakSet Object ---");
    const ws = new WeakSet();
    const val1 = { id: 1 };
    const val2 = { id: 2 };

    ws.add(val1);
    ws.add(val2);

    assertEqual(ws.has(val1), true, "WeakSet has val1");
    assertEqual(ws.has({ id: 1 }), false, "WeakSet does not have different object reference");

    assertEqual(ws.delete(val1), true, "WeakSet delete val1");
    assertEqual(ws.has(val1), false, "WeakSet has val1 after delete");
    assertEqual(ws.delete(val1), false, "WeakSet delete val1 again");

    const val3 = { id: 3 };
    const ws2 = new WeakSet([val3]);
    assertEqual(ws2.has(val3), true, "WeakSet2 initialized with val3");
}

function main() {
    console.log("=== RUNNING ECMASCRIPT BUILT-INS TEST SUITE ===");
    runGlobalFunctionsTests();
    runDateTests();
    runMapTests();
    runSetTests();
    runJSONTests();
    runRegExpTests();
    runObjectTests();
    runNumberTests();
    runWeakMapTests();
    runWeakSetTests();
    console.log("=== ALL BUILT-IN TESTS PASSED SUCCESSFULLY ===");
}

main();
