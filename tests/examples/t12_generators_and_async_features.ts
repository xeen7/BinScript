function assertEqual(actual: any, expected: any, description: string) {
  const result = actual === expected;
  console.log("Assert [" + description + "]: expected " + expected + ", got " + actual + " → " + (result ? "PASS" : "FAIL"));
  if (!result) {
    throw new Error("Assertion failed: " + description);
  }
}

function logged(target: any, key: string, descriptor?: any) {
  return descriptor;
}

class UserProfile {
  name: string;
  role: string;

  constructor(name: string, role: string) {
    this.name = name;
    this.role = role;
  }

  @logged
  getDetails() {
    return this.name + " (" + this.role + ")";
  }
}

type Point = { x: number; y: number };
type Point3D = Point & { z: number }; 

type ReadonlyPoint = { readonly [P in keyof Point]: Point[P] }; 
type IsString<T> = T extends string ? true : false; 
type InferElement<T> = T extends (infer U)[] ? U : T; 

function* fibonacci(limit: number) {
  let a = 0;
  let b = 1;
  let count = 0;
  while (count < limit) {
    yield a;
    let next = a + b;
    a = b;
    b = next;
    count = count + 1;
  }
}

function* innerGen() {
  yield 100;
  yield 200;
}

function* outerGen() {
  yield* innerGen(); 
  yield 300;
}

async function* asyncCountdown(start: number) {
  let current = start;
  while (current > 0) {
    yield current;
    current = current - 1;
  }
}

async function runAsyncLoop() {
  let sum = 0;
  const gen = asyncCountdown(3);
  for await (const x of gen) {
    sum = sum + x;
  }
  return sum;
}

async function runTests() {
  console.log("=== RUNNING GENERATORS & ASYNC COMPILER TEST SUITE ===");

  const user = new UserProfile("Alice", "Admin");
  assertEqual(user.getDetails(), "Alice (Admin)", "Decorator method invocation");

  console.log("\n--- Testing standard generator (Fibonacci) ---");
  const fib = fibonacci(5);
  assertEqual(fib.next().value, 0, "Fibonacci term 1");
  assertEqual(fib.next().value, 1, "Fibonacci term 2");
  assertEqual(fib.next().value, 1, "Fibonacci term 3");
  assertEqual(fib.next().value, 2, "Fibonacci term 4");
  assertEqual(fib.next().value, 3, "Fibonacci term 5");
  assertEqual(fib.next().done, true, "Fibonacci finished");

  console.log("\n--- Testing yield* delegation generator ---");
  const out = outerGen();
  assertEqual(out.next().value, 100, "yield* term 1");
  assertEqual(out.next().value, 200, "yield* term 2");
  assertEqual(out.next().value, 300, "outer term 3");
  assertEqual(out.next().done, true, "outer finished");

  console.log("\n--- Testing async generators & for await...of loops ---");
  const asyncResultPromise = runAsyncLoop();

  const finalSum = await asyncResultPromise;
  assertEqual(finalSum, 6, "Sum of async countdown 3 + 2 + 1");

  console.log("\n=== ALL RECENTLY ACHIEVED FEATURES COMPLETED SUCCESSFULLY ===");
}

async function main() {
  const promise = runTests();
  await promise;
}

main();
