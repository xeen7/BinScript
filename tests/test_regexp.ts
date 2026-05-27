const r1 = /hello/gi;
console.log(r1.source); // "hello"
console.log(r1.flags);  // "gi"

const r2 = new RegExp("world", "m");
console.log(r2.source); // "world"
console.log(r2.flags);  // "m"
