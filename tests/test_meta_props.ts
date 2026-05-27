const nt = new.target;
console.log(nt); // undefined

const im = import.meta;
console.log(im.url); // "file:///main.ts"
