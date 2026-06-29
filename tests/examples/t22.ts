function* myGen() {
  yield 42;
}
const g = myGen();
console.log(g.next().value);
console.log(g.next().done);
