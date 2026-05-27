const assert = (cond: boolean, msg: string) => {
    if (!cond) throw new Error("Assertion failed: " + msg);
};

// 1. Exponentiation
assert(2 ** 3 === 8, "2 ** 3 == 8");
assert(3 ** 2 === 9, "3 ** 2 == 9");

// 2. Unary Plus
assert(+"42" === 42, "+'42' == 42");
assert(+true === 1, "+true == 1");
assert(+false === 0, "+false == 0");
assert(+null === 0, "+null == 0");

// 3. Nullish Coalescing
const a = null ?? "default";
assert(a === "default", "null ?? default");
const b = undefined ?? "default";
assert(b === "default", "undefined ?? default");
const c = 0 ?? "default";
assert(c === 0, "0 ?? default");
const d = false ?? true;
assert(d === false, "false ?? true");
const e = "" ?? "non-empty";
assert(e === "", "empty string ?? non-empty");

// 4. In operator
const obj = { x: 10, y: 20 };
assert("x" in obj === true, "'x' in obj");
assert("z" in obj === false, "'z' in obj");
// Wait, the test runner needs `console.log` to print success
console.log("All operators passed!");

// 5. Delete operator
delete obj.x;
assert("x" in obj === false, "deleted 'x' in obj");
assert("y" in obj === true, "'y' in obj");

console.log("Delete passed!");
