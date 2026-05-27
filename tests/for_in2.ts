let obj = { x: 100, y: 200 };
let k = "test";
for (k in obj) {
    console.log(k);
}
console.log("last k is:", k);
