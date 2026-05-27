console.log("=== ES2022 STDLIB & TYPEOF COMPLETE TEST ===");

// 1. typeof Operator
console.log(typeof undefined); // "undefined"
console.log(typeof null); // "object"
console.log(typeof true); // "boolean"
console.log(typeof 42); // "number"
console.log(typeof "hello"); // "string"
console.log(typeof []); // "object"
console.log(typeof {}); // "object"
console.log(typeof new Date()); // "object"

// 2. Global constants & static Number fields
console.log(isNaN(NaN)); // true
console.log(isFinite(Infinity)); // false

console.log(isNaN(Number.NaN)); // true
console.log(Number.POSITIVE_INFINITY > 0); // true
console.log(Number.NEGATIVE_INFINITY < 0); // true
console.log(Number.MAX_VALUE > 0); // true
console.log(Number.MIN_VALUE > 0); // true
console.log(Number.EPSILON > 0); // true

// 3. Object Static Methods
const target = { a: 1 };
const source = { b: 2, c: 3 };
Object.assign(target, source);
console.log(target.a); // 1
console.log(target.b); // 2
console.log(target.c); // 3

const protoObj = { protoField: 42 };
const createdObj = Object.create(protoObj);
console.log(createdObj.protoField); // 42
console.log(Object.getPrototypeOf(createdObj) === protoObj); // true

const sampleObj = { x: 10, y: 20 };
const keys = Object.keys(sampleObj);
console.log(keys.length); // 2
console.log(keys.includes("x")); // true
console.log(keys.includes("y")); // true

const values = Object.values(sampleObj);
console.log(values.length); // 2
console.log(values.includes(10)); // true
console.log(values.includes(20)); // true

const entries = Object.entries(sampleObj);
console.log(entries.length); // 2
console.log(entries[0].length); // 2

// 4. String Static Methods
console.log(String.fromCharCode(65)); // "A"
console.log(String.fromCodePoint(97)); // "a"

// 5. Primitive Wrapper Constructors & Coercions
const strWrap = new String("hello wrapper");
console.log(typeof strWrap); // "object"
console.log(strWrap.length); // 13
console.log(strWrap.toUpperCase()); // "HELLO WRAPPER"
console.log(strWrap.charAt(1)); // "e"
console.log(strWrap.indexOf("wrapper")); // 6

const numWrap = new Number(123.45);
console.log(typeof numWrap); // "object"
console.log(numWrap.valueOf()); // 123.45
console.log(numWrap.toString()); // "123.45"

const boolWrap = new Boolean(true);
console.log(typeof boolWrap); // "object"
console.log(boolWrap.valueOf()); // true
console.log(boolWrap.toString()); // "true"

// 6. Date instance methods
const date = new Date("2026-05-21T15:30:45");
console.log(date.getFullYear()); // 2026
console.log(date.getMonth()); // 4
console.log(date.getDate()); // 21
console.log(date.getHours()); // 15
console.log(date.getMinutes()); // 30
console.log(date.getSeconds()); // 45
console.log(date.getTime() > 0); // true

// 7. Object.fromEntries
const kvPairs = [["foo", "bar"], ["baz", 42]];
const entryObj = Object.fromEntries(kvPairs);
console.log(entryObj.foo); // "bar"
console.log(entryObj.baz); // 42
console.log(typeof entryObj); // "object"

// 8. globalThis
console.log(globalThis === undefined); // false
console.log(typeof globalThis); // "object"
globalThis.myGlobalVar = 999;
console.log(globalThis.myGlobalVar); // 999

// 9. URI encoding/decoding
const uri = "http://example.com/a b?c=d&e=f";
const encodedURI = encodeURI(uri);
console.log(encodedURI); // "http://example.com/a%20b?c=d&e=f"
const decodedURI = decodeURI(encodedURI);
console.log(decodedURI); // "http://example.com/a b?c=d&e=f"

const component = "a b?c=d&e=f";
const encodedComp = encodeURIComponent(component);
console.log(encodedComp); // "a%20b%3Fc%3Dd%26e%3Df"
const decodedComp = decodeURIComponent(encodedComp);
console.log(decodedComp); // "a b?c=d&e=f"

// 10. URIError handling
try {
    decodeURIComponent("%Z1");
    console.log("Failed to throw URIError");
} catch (e) {
    console.log(e.name); // "URIError"
    console.log(e.message); // "URI malformed"
}

// 11. Refinement tests (0 vs 1 vs n arguments)
console.log("--- Refinement Tests ---");

// String 0 vs 1
const s0 = new String();
const s1 = new String(undefined);
console.log(s0.valueOf() === ""); // true
console.log(s1.valueOf() === "undefined"); // true

// Number 0 vs 1
const n0 = new Number();
const n1 = new Number(undefined);
console.log(n0.valueOf() === 0); // true
console.log(isNaN(n1.valueOf())); // true

// Boolean 0 vs 1
const b0 = new Boolean();
const b1 = new Boolean(undefined);
console.log(b0.valueOf() === false); // true
console.log(b1.valueOf() === false); // true

// Coercions (regular function call)
console.log(String() === ""); // true
console.log(String(undefined) === "undefined"); // true
console.log(Number() === 0); // true
console.log(isNaN(Number(undefined))); // true
console.log(Boolean() === false); // true
console.log(Boolean(undefined) === false); // true
console.log(typeof Date()); // "string"
console.log(typeof Date("some arg")); // "string"

const objCoerce0 = Object();
const objCoerce1 = Object(42);
console.log(typeof objCoerce0); // "object"
console.log(typeof objCoerce1); // "object"
console.log(objCoerce1.valueOf() === 42); // true

// Date 0 vs 1 vs n
const d0 = new Date();
console.log(d0.getTime() > 0); // true

const d1 = new Date(1716297045000);
console.log(d1.getFullYear() > 2020); // true

// multi-argument Date(y, m, d, h, min, s, ms)
const dN = new Date(2026, 4, 21, 15, 30, 45, 123);
console.log(dN.getFullYear()); // 2026
console.log(dN.getMonth()); // 4
console.log(dN.getDate()); // 21
console.log(dN.getHours()); // 15
console.log(dN.getMinutes()); // 30
console.log(dN.getSeconds()); // 45

// 2-digit year mapping (99 -> 1999)
const d2Digit = new Date(99, 5, 12);
console.log(d2Digit.getFullYear()); // 1999

console.log("=== ALL TESTS COMPLETED SUCCESSFULLY ===");
