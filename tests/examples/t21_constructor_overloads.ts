function assertEqual(actual: any, expected: any, message: string) {
    if (actual !== expected) {
        throw new Error("Assert [" + message + "]: expected " + expected + ", got " + actual + " → FAIL");
    }
    console.log("Assert [" + message + "]: expected " + expected + ", got " + actual + " → PASS");
}

function runDateConstructorTests() {
    console.log("--- Date Constructor ---");
    const d1 = new Date();
    assertEqual(typeof d1.getTime() === "number" && d1.getTime() > 0, true, "new Date()");

    const d2 = new Date(1715990400000);
    assertEqual(d2.getTime(), 1715990400000, "new Date(timestamp)");

    const d3 = new Date("2024/05/18");
    assertEqual(d3.getTime() > 0, true, "new Date(dateString)");

    const d4 = new Date(2024, 4, 18, 12, 30, 0, 0);
    assertEqual(d4.getFullYear(), 2024, "new Date(multiple args) year");
    assertEqual(d4.getMonth(), 4, "new Date(multiple args) month");
}

function runErrorConstructorTests() {
    console.log("--- Error Constructor ---");
    const err1 = new Error();
    assertEqual(err1.message, "", "new Error()");

    const err2 = new Error("custom message");
    assertEqual(err2.message, "custom message", "new Error(message)");
}

function runMapSetConstructorTests() {
    console.log("--- Map & Set Constructors ---");
    const m1 = new Map();
    assertEqual(m1.size, 0, "new Map()");

    const m2 = new Map([["key", "value"]]);
    assertEqual(m2.get("key"), "value", "new Map(iterable)");

    const s1 = new Set();
    assertEqual(s1.size, 0, "new Set()");

    const s2 = new Set(["value"]);
    assertEqual(s2.has("value"), true, "new Set(iterable)");
}

function runWeakMapSetConstructorTests() {
    console.log("--- WeakMap & WeakSet Constructors ---");
    const wm1 = new WeakMap();
    assertEqual(wm1 instanceof WeakMap, true, "new WeakMap()");

    const k = { id: 1 };
    const wm2 = new WeakMap([[k, "value"]]);
    assertEqual(wm2.get(k), "value", "new WeakMap(iterable)");

    const ws1 = new WeakSet();
    assertEqual(ws1 instanceof WeakSet, true, "new WeakSet()");

    const ws2 = new WeakSet([k]);
    assertEqual(ws2.has(k), true, "new WeakSet(iterable)");
}

function main() {
    console.log("=== RUNNING CONSTRUCTOR OVERLOADS TEST SUITE ===");
    runDateConstructorTests();
    runErrorConstructorTests();
    runMapSetConstructorTests();
    runWeakMapSetConstructorTests();
    console.log("=== ALL CONSTRUCTOR OVERLOADS TESTS PASSED SUCCESSFULLY ===");
}

main();
