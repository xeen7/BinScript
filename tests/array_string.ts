// Array LITERALS & INDEXING
let arr = [1, 2, 3];
console.log(arr.length); // 3
console.log(arr[0]); // 1
console.log(arr[1]); // 2
console.log(arr[2]); // 3

// INDEX SETTING
arr[1] = 42;
console.log(arr[1]); // 42
console.log(arr.length); // 3

// PUSH & POP
let newLen = arr.push(100);
console.log(newLen); // 4
console.log(arr[3]); // 100
console.log(arr.pop()); // 100

// HIGHER-ORDER METHODS
let mapped = arr.map((x: number) => x * 2);
console.log(mapped[0]); // 2
console.log(mapped[1]); // 84
console.log(mapped[2]); // 6

let filtered = arr.filter((x: number) => x > 2);
console.log(filtered.length); // 1
console.log(filtered[0]); // 42

let sum = arr.reduce((acc: number, x: number) => acc + x, 0);
console.log(sum); // 46 (1 + 42 + 3)

// INCLUDES & INDEXOF
console.log(arr.includes(42)); // true
console.log(arr.indexOf(42)); // 1
console.log(arr.indexOf(999)); // -1

// STRING METHODS
let s = "  hello world  ";
console.log(s.length); // 15
let trimmed = s.trim();
console.log(trimmed); // "hello world"
console.log(trimmed.toUpperCase()); // "HELLO WORLD"
console.log(trimmed.charAt(1)); // "e"
console.log(trimmed.charCodeAt(1)); // 101

let parts = trimmed.split(" ");
console.log(parts.length); // 2
console.log(parts[0]); // "hello"
console.log(parts[1]); // "world"
