function traverse(obj: any, depth: number) {
    if (depth == 0) return obj.val;
    return traverse(obj, depth - 1) + 1;
}

function testRecursion() {
    let sum = 0;
    for (let i = 0; i < 100; i++) {
        let node = { val: i };
        sum += traverse(node, 10);
    }
    console.log("Recursion complete! Final sum: " + sum);
}

testRecursion();
