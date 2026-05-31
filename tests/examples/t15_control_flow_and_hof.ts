function assertEqual(name: string, actual: any, expected: any) {
  if (actual === expected) {
    console.log(`Assert [${name}]: expected ${expected}, got ${actual} → PASS`);
  } else {
    console.error(`Assert [${name}]: expected ${expected}, got ${actual} → FAIL`);
    throw new Error(`Assertion failed: ${name}`);
  }
}

function makeAdder(base: number) {
  return (x: number) => base + x;
}

function makeAccumulator(start: number) {
  let count = start;
  return () => { count += 1; return count; };
}

function testClosures() {
  const add5 = makeAdder(5);
  assertEqual("Adder 5+3", add5(3), 8.0);
  assertEqual("Adder 5+10", add5(10), 15.0);

  const inc = makeAccumulator(10);
  assertEqual("Acc inc 1", inc(), 11.0);
  assertEqual("Acc inc 2", inc(), 12.0);
  assertEqual("Acc inc 3", inc(), 13.0);
}

function testTryCatch() {
  let log = "";

  try {
    log += "try";
    throw new Error("oops");
  } catch (e: any) {
    log += "-catch";
  } finally {
    log += "-finally";
  }

  assertEqual("Try-catch-finally log", log, "try-catch-finally");

  let safe = "";
  try {
    safe = "ok";
  } catch (e: any) {
    safe = "fail";
  }
  assertEqual("No-throw try", safe, "ok");
}

function grade(score: number): string {
  switch (true) {
    case score >= 90: return "A";
    case score >= 80: return "B";
    case score >= 70: return "C";
    default:          return "F";
  }
}

function testSwitch() {
  assertEqual("Grade 95", grade(95), "A");
  assertEqual("Grade 82", grade(82), "B");
  assertEqual("Grade 71", grade(71), "C");
  assertEqual("Grade 50", grade(50), "F");
}

function map(arr: any[], fn: any): any[] {
  const result: any[] = [];
  for (const item of arr) {
    result.push(fn(item));
  }
  return result;
}

function filter(arr: any[], fn: any): any[] {
  const result: any[] = [];
  for (const item of arr) {
    if (fn(item)) result.push(item);
  }
  return result;
}

function reduce(arr: any[], fn: any, init: any): any {
  let acc = init;
  for (const item of arr) {
    acc = fn(acc, item);
  }
  return acc;
}

function testHigherOrder() {
  const nums = [1, 2, 3, 4, 5];

  const doubled = map(nums, (x: any) => x * 2);
  assertEqual("Map doubled[0]", doubled[0], 2.0);
  assertEqual("Map doubled[4]", doubled[4], 10.0);

  const evens = filter(nums, (x: any) => x % 2 === 0);
  assertEqual("Filter evens length", evens.length, 2.0);
  assertEqual("Filter evens[0]", evens[0], 2.0);

  const sum = reduce(nums, (acc: any, x: any) => acc + x, 0);
  assertEqual("Reduce sum", sum, 15.0);
}

function testStrings() {
  const s = "  Hello, World!  ";
  const trimmed = s.trim();
  assertEqual("Trim", trimmed, "Hello, World!");

  const upper = "hello".toUpperCase();
  assertEqual("toUpperCase", upper, "HELLO");

  const lower = "WORLD".toLowerCase();
  assertEqual("toLowerCase", lower, "world");

  const replaced = "foo bar foo".replace("foo", "baz");
  assertEqual("Replace first", replaced, "baz bar foo");

  const parts = "a,b,c".split(",");
  assertEqual("Split length", parts.length, 3.0);
  assertEqual("Split[1]", parts[1], "b");

  const joined = parts.join("-");
  assertEqual("Join", joined, "a-b-c");
}

function testBitwise() {
  const a = 10;  
  const b = 12;  

  assertEqual("AND",  (a & b),   8.0);
  assertEqual("OR",   (a | b),  14.0);
  assertEqual("XOR",  (a ^ b),   6.0);
  assertEqual("Left shift",  (1 << 3),  8.0);
  assertEqual("Right shift", (16 >> 2), 4.0);
}

function testNullish() {
  const a: any = null;
  const b: any = undefined;
  const c: any = "hello";

  assertEqual("Nullish a ?? fallback",   a ?? "fallback", "fallback");
  assertEqual("Nullish b ?? fallback",   b ?? "fallback", "fallback");
  assertEqual("Nullish c ?? fallback",   c ?? "fallback", "hello");
  assertEqual("Optional a?.length",      a?.length, undefined);
  assertEqual("Optional c?.length",      c?.length, 5.0);
  assertEqual("NullishAssign ??=",       (a ?? "assigned"), "assigned");
}

function testForIn() {
  const obj: any = { x: 1, y: 2, z: 3 };
  let keys: any[] = [];
  for (const k in obj) {
    keys.push(k);
  }
  assertEqual("for..in length", keys.length, 3.0);
}

function main() {
  console.log("=== RUNNING CONTROL FLOW & HIGHER-ORDER FUNCTIONS TEST SUITE ===");

  testClosures();
  testTryCatch();
  testSwitch();
  testHigherOrder();
  testStrings();
  testBitwise();
  testNullish();
  testForIn();

  console.log("=== ALL TESTS COMPLETED SUCCESSFULLY ===");
}

main();
