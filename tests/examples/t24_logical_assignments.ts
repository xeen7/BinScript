let counter = 0;
function getObj() {
    counter++;
    return { prop: 10 };
}

let obj1 = getObj();
obj1.prop &&= 20;

let obj2 = { prop: 0 };
obj2.prop ||= 30;

let obj3 = { prop: null };
obj3.prop ??= 40;

console.log("Counter:", counter);
console.log("obj1:", obj1.prop);
console.log("obj2:", obj2.prop);
console.log("obj3:", obj3.prop);

let shortCircuitCounter = 0;
function getFalsy() {
    return { prop: 0 };
}
getFalsy().prop &&= (shortCircuitCounter = 100);

console.log("shortCircuitCounter:", shortCircuitCounter);
