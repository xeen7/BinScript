function main() {
  let original = { name: "samon", age: "25" };
  let result = new Object(original);

  console.log(original === result);
  console.log(`hello world ! , your name is : ${original.age} your age is : ${original.age}`);
}
main();
