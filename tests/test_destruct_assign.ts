let a = 1;
let b = 2;
let c = 3;

console.log("--- 1. Array destructuring assignment ---");
const arr = [10, 20, 30];
const res1 = ([a, b, c] = arr);
console.log(a); // 10
console.log(b); // 20
console.log(c); // 30
console.log("Evaluation value:");
console.log(res1[0]); // 10
console.log(res1[1]); // 20

console.log("--- 2. Object destructuring assignment ---");
const obj = { x: 100, y: 200 };
const res2 = ({ x: a, y: b } = obj);
console.log(a); // 100
console.log(b); // 200
console.log("Evaluation value:");
console.log(res2.x); // 100

console.log("--- 3. Nested destructuring assignment ---");
const nested = [500, { z: 600 }];
[a, { z: b }] = nested;
console.log(a); // 500
console.log(b); // 600

console.log("--- 4. Member expression targets ---");
const myObj = { val1: 0, val2: 0 };
[myObj.val1, myObj.val2] = [777, 888];
console.log(myObj.val1); // 777
console.log(myObj.val2); // 888

console.log("--- 5. Default values ---");
let d = 5;
[d = 99] = [];
console.log(d); // 99

let e = 6;
[e = 99] = [123];
console.log(e); // 123
