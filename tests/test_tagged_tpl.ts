function tag(strings: string[], name: string, age: number): string {
  console.log(strings[0]); // "Hello "
  console.log(strings[1]); // " you are "
  console.log(strings[2]); // " years old."
  console.log(name);       // "Alice"
  console.log(age);        // 30
  return "result";
}

const name = "Alice";
const age = 30;
const res = tag`Hello ${name} you are ${age} years old.`;
console.log(res); // "result"
