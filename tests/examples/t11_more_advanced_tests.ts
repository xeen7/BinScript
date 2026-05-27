function assertEqual(actual: any, expected: any, description: string) {
  const result = actual === expected;
  console.log("Assert [" + description + "]: expected " + expected + ", got " + actual + " → " + (result ? "PASS" : "FAIL"));
  if (!result) {
    throw new Error("Assertion failed: " + description);
  }
}

function main() {
  console.log("=== RUNNING ADVANCED COMPILER SUITE ===");

  console.log("\n--- 1. Testing Dynamic typeof ---");
  assertEqual(typeof 42, "number", "typeof number");
  assertEqual(typeof "hello", "string", "typeof string");
  assertEqual(typeof true, "boolean", "typeof boolean");
  assertEqual(typeof null, "object", "typeof null");
  assertEqual(typeof [1, 2, 3], "object", "typeof array");
  assertEqual(typeof { a: 1 }, "object", "typeof object");

  const myFunc = (x: number) => x * 2;
  assertEqual(typeof myFunc, "function", "typeof closure function");

  console.log("\n--- 2. Testing Logical & Nullish Coalescing ---");
  const nullVal: any = null;
  const undefVal: any = undefined;
  const definedVal = "hello";

  assertEqual(nullVal ?? "fallback", "fallback", "null ?? fallback");
  assertEqual(undefVal ?? "fallback", "fallback", "undefined ?? fallback");
  assertEqual(definedVal ?? "fallback", "hello", "defined ?? fallback");

  assertEqual(true && "right", "right", "true && right");
  assertEqual(false && "right", false, "false && right");
  const leftStr = "left";
  assertEqual(leftStr || "right", "left", "string || right");
  assertEqual(false || "right", "right", "false || right");

  console.log("\n--- 3. Testing Optional Chaining & Objects ---");
  const nestedObj: any = {
    nested: {
      value: 100,
      method: (x: number) => x + 150
    }
  };

  assertEqual(nestedObj?.nested?.value, 100, "nestedObj?.nested?.value");
  assertEqual(nestedObj?.nested?.method?.(10), 160, "nestedObj?.nested?.method?.(10)");
  assertEqual(nestedObj?.nonExistent?.prop, undefined, "nestedObj?.nonExistent?.prop");
  assertEqual(nestedObj?.nonExistent?.method?.(), undefined, "nestedObj?.nonExistent?.method?.()");

  console.log("\n--- 4. Testing Array Operations & Spreads ---");
  const arr = [1, 2, 3];
  assertEqual(arr.length, 3, "initial array length");

  arr.push(4);
  assertEqual(arr.length, 4, "length after push");
  assertEqual(arr.pop(), 4, "popped value");
  assertEqual(arr.length, 3, "length after pop");

  const arr2 = [0, ...arr, 4];
  assertEqual(arr2.length, 5, "spread array length");
  assertEqual(arr2[0], 0, "spread array [0]");
  assertEqual(arr2[1], 1, "spread array [1]");
  assertEqual(arr2[4], 4, "spread array [4]");

  console.log("\n--- 5. Testing Lexical Scoping Mutability ---");
  let sharedState = { count: 0 };
  const incrementer = () => {
    sharedState.count = sharedState.count + 1;
    return sharedState.count;
  };
  const decrementer = () => {
    sharedState.count = sharedState.count - 1;
    return sharedState.count;
  };

  assertEqual(incrementer(), 1, "incrementer first run");
  assertEqual(incrementer(), 2, "incrementer second run");
  assertEqual(decrementer(), 1, "decrementer first run");
  assertEqual(sharedState.count, 1, "final count inside box");

  console.log("\n=== ALL ADVANCED TESTS COMPLETED SUCCESSFULLY ===");
}
main();
