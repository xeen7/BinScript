// Math Constants
console.log(Math.PI > 3.14 && Math.PI < 3.15); // true
console.log(Math.E > 2.71 && Math.E < 2.72); // true

// Math Functions
console.log(Math.floor(3.7)); // 3
console.log(Math.ceil(3.2)); // 4
console.log(Math.round(3.5)); // 4
console.log(Math.round(3.4)); // 3
console.log(Math.abs(-42)); // 42
console.log(Math.sqrt(16)); // 4
console.log(Math.pow(2, 3)); // 8
console.log(Math.min(10, 20)); // 10
console.log(Math.max(10, 20)); // 20
console.log(Math.trunc(3.9)); // 3

// Trigo
console.log(Math.sin(0)); // 0
console.log(Math.cos(0)); // 1

// Random
let r = Math.random();
console.log(r >= 0 && r < 1); // true

// Globals
console.log(parseInt("42")); // 42
console.log(parseInt("1010", 2)); // 10
console.log(parseInt("2a", 16)); // 42
console.log(parseFloat("3.14")); // 3.14

// isNaN
console.log(isNaN(NaN)); // true
console.log(isNaN(42)); // false
console.log(isNaN("hello")); // true

// isFinite
console.log(isFinite(42)); // true
console.log(isFinite(Infinity)); // false

// Number.isInteger
console.log(Number.isInteger(42)); // true
console.log(Number.isInteger(42.5)); // false
