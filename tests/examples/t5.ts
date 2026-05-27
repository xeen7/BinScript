function main() {
  let a = 1;
  let b = "hello";
  let c = [5, 4, 4, 55];
  let d = { name: "samon", age: 26, id: 123, height: 5.9, isStudent: true, data: [1, 2, 3] };
  c[1] = c[0] + 6 + 8 + 7;
  console.log(typeof c);
  console.log(typeof d);
  console.log(d.name);
  console.log(d.age);
  console.log(c[1]);
}
main();
