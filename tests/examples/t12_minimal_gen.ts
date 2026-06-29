// Minimal generator test
function* simple() {
  yield 1;
  yield 2;
}

const g = simple();
const r1 = g.next();
console.log(r1.value);
const r2 = g.next();
console.log(r2.value);
console.log("done");
