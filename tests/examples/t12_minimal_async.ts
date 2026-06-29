// Minimal async test
async function hello() {
  console.log("hello async");
  return 42;
}

async function main() {
  const r = await hello();
  console.log(r);
}

main();
