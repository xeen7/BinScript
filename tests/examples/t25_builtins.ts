let arr = [10, 20, 30];
console.log("arr.at(1):", arr.at(1));
console.log("arr.at(-1):", arr.at(-1));
console.log("arr.at(-2):", arr.at(-2));
console.log("arr.at(5):", arr.at(5)); // undefined
console.log("arr.at(-5):", arr.at(-5)); // undefined

let str = "hello";
console.log("str.at(1):", str.at(1));
console.log("str.at(-1):", str.at(-1));
console.log("str.at(-2):", str.at(-2));
console.log("str.at(5):", str.at(5)); // undefined
console.log("str.at(-5):", str.at(-5)); // undefined

let obj = { x: 100, y: 200 };
console.log("hasOwn(obj, x):", Object.hasOwn(obj, "x"));
console.log("hasOwn(obj, y):", Object.hasOwn(obj, "y"));
console.log("hasOwn(obj, z):", Object.hasOwn(obj, "z"));
console.log("hasOwn(obj, toString):", Object.hasOwn(obj, "toString"));
