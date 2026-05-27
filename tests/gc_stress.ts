// Test GC stress
// Allocates a large number of objects in a loop to ensure GC runs and reclaims memory.

function stress_test() {
    let i = 0;
    while (i < 50000) {
        // Allocate objects that will become garbage
        let obj1 = { a: i };
        let obj2 = { b: i, c: obj1 };
        i = i + 1;
    }
    console.log("Stress test done. i is:");
    console.log(i);
}

stress_test();
