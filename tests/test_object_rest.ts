const obj = { a: 1, b: 2, c: 3, d: 4 };

// Let rest destructuring
const { a, b, ...rest } = obj;
console.log(a);       // 1
console.log(b);       // 2
console.log(rest.c);  // 3
console.log(rest.d);  // 4
console.log(rest.a);  // undefined

// Assignment rest destructuring
let x: number = 0;
let y: number = 0;
let remaining: any = null;
({ a: x, c: y, ...remaining } = obj);
console.log(x);           // 1
console.log(y);           // 3
console.log(remaining.b); // 2
console.log(remaining.d); // 4
console.log(remaining.a); // undefined
