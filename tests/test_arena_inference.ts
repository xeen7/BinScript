// Test: Arena Grouping Inference
// Two objects allocated inside process() never escape the function.
// The compiler should detect this and use ArenaAlloc + ArenaDestroy.

class Point {
    x: number;
    y: number;

    constructor(x: number, y: number) {
        this.x = x;
        this.y = y;
    }
}

function process(): number {
    let p1 = new Point(10, 20);
    let p2 = new Point(30, 40);
    let sum = p1.x + p2.y;
    return sum;
}

let result = process();
console.log(result); // Expected: 50
