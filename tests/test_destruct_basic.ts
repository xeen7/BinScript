// Basic and Nested Destructuring Test Suite

// 1. Array destructuring
console.log("--- 1. Array destructuring ---");
const [a, b] = [10, 20];
console.log(a); // 10
console.log(b); // 20

// 2. Sparse array/skips
console.log("--- 2. Sparse array/skips ---");
const [c, , d] = [30, 40, 50];
console.log(c); // 30
console.log(d); // 50

// 3. Array rest elements
console.log("--- 3. Array rest elements ---");
const [e, ...f] = [60, 70, 80];
console.log(e); // 60
console.log(f.length); // 2
console.log(f[0]); // 70
console.log(f[1]); // 80

// 4. Object destructuring (shorthand)
console.log("--- 4. Object destructuring (shorthand) ---");
const { x, y } = { x: 100, y: 200 };
console.log(x); // 100
console.log(y); // 200

// 5. Object rename
console.log("--- 5. Object rename ---");
const { x: aRename, y: bRename } = { x: 300, y: 400 };
console.log(aRename); // 300
console.log(bRename); // 400

// 6. Object default values
console.log("--- 6. Object default values ---");
const { u = 99, v = 88 } = { u: 11 };
console.log(u); // 11
console.log(v); // 88

// 7. Nested destructuring
console.log("--- 7. Nested destructuring ---");
const { p: { q: nestedVal } } = { p: { q: 500 } };
console.log(nestedVal); // 500
