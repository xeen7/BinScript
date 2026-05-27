class Point {
    x: number;
    y: number;
    constructor(x: number, y: number) {
        this.x = x;
        this.y = y;
    }
    sum(): number {
        return this.x + this.y;
    }
}
const p = new Point(10, 20);
console.log(p.x);      // Output: 10
console.log(p.sum());  // Output: 30
p.x = 40;
console.log(p.sum());  // Output: 60
