console.log("Before await");
let p = Promise.resolve("Top Level Await!");
let res = await p;
console.log(res);
