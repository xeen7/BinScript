// Labeled Statements (Loops and Blocks) Test Suite

// 1. Nested loops with break label
console.log("--- 1. Nested loops with break label ---");
let count1 = 0;
outer1: for (let i = 0; i < 3; i++) {
    for (let j = 0; j < 3; j++) {
        if (i === 1 && j === 1) {
            break outer1;
        }
        count1++;
    }
}
console.log(count1); // Should print 4 (i=0,j=0; i=0,j=1; i=0,j=2; i=1,j=0)

// 2. Nested loops with continue label
console.log("--- 2. Nested loops with continue label ---");
let count2 = 0;
outer2: for (let i = 0; i < 3; i++) {
    for (let j = 0; j < 3; j++) {
        if (j === 1) {
            continue outer2;
        }
        count2++;
    }
}
console.log(count2); // Should print 3 (i=0,j=0; i=1,j=0; i=2,j=0)

// 3. Labeled block break
console.log("--- 3. Labeled block break ---");
let val = 0;
my_block: {
    val = 10;
    if (val === 10) {
        break my_block;
    }
    val = 20; // skipped
}
console.log(val); // Should print 10

// 4. Labeled switch break
console.log("--- 4. Labeled switch break ---");
let switchVal = 0;
let x = 1;
my_switch: switch (x) {
    case 1:
        switchVal = 42;
        break my_switch;
        switchVal = 99; // skipped
}
console.log(switchVal); // Should print 42

// 5. Labeled for-in loop break & continue
console.log("--- 5. Labeled for-in loop ---");
let obj: any = { a: 1, b: 2, c: 3 };
let keysCount = 0;
outer_for_in: for (let k in obj) {
    if (k === "b") {
        continue outer_for_in;
    }
    keysCount++;
}
console.log(keysCount); // Should print 2 (a, c)
