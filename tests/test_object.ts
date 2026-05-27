// ============================================================================
// Advanced Test Suite: Object — Primitive Wrapper & Dynamic Properties
// ============================================================================

console.log("=== TEST: Object ===");

// --- 1. Object Literals with Various Value Types ---
console.log("--- 1. Mixed-type literals ---");
const mixed = { num: 42, str: "hello", flag: true, nothing: null };
console.log(mixed.num);      // 42
console.log(mixed.str);      // hello
console.log(mixed.flag);     // true
console.log(mixed.nothing);  // null

// --- 2. Nested Object Access ---
console.log("--- 2. Nested access ---");
const outer = { inner: { deep: { value: 999 } } };
console.log(outer.inner.deep.value); // 999

// --- 3. Dynamic Property Assignment ---
console.log("--- 3. Dynamic properties ---");
const obj = new Object();
obj.x = 10;
obj.y = 20;
obj.label = "point";
console.log(obj.x);     // 10
console.log(obj.y);     // 20
console.log(obj.label);  // point

// --- 4. Property Overwrite ---
console.log("--- 4. Overwrite ---");
obj.x = 100;
console.log(obj.x); // 100

// --- 5. Object as Function Argument ---
console.log("--- 5. Pass by reference ---");
function setName(o: any, name: string) {
    o.name = name;
}
const person = new Object();
setName(person, "Alice");
console.log(person.name); // Alice

// --- 6. Object Chaining via Properties ---
console.log("--- 6. Object chain ---");
const a = new Object();
const b = new Object();
const c = new Object();
a.next = b;
b.next = c;
c.value = 777;
console.log(a.next.next.value); // 777

// --- 7. Printing Objects (no segfault) ---
console.log("--- 7. Print objects ---");
console.log(mixed);   // Object {}
console.log(obj);     // Object {}
console.log(person);  // Object {}

// --- 8. Object Literal with Computed Numeric Keys ---
console.log("--- 8. Numeric-like fields ---");
const config = { width: 1920, height: 1080, fps: 60 };
const total = config.width * config.height;
console.log(total);           // 2073600
console.log(config.fps);      // 60

// --- 9. Deeply Nested Property Mutation ---
console.log("--- 9. Deep mutation ---");
const root = { child: { leaf: 0 } };
root.child.leaf = 42;
console.log(root.child.leaf); // 42

// --- 10. Object as Accumulator ---
console.log("--- 10. Accumulator pattern ---");
const stats = new Object();
stats.count = 0;
stats.total = 0;
const values = [10, 20, 30, 40];
for (let i = 0; i < 4; i = i + 1) {
    stats.count = stats.count + 1;
    stats.total = stats.total + values[i];
}
console.log(stats.count); // 4
console.log(stats.total); // 100

console.log("=== OBJECT TESTS PASSED ===");
