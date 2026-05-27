// Spread in Calls Test Suite

function sum(a: number, b: number, c: number): number {
    return a + b + c;
}

let args = [2, 3];
console.log("Testing sum(1, ...[2, 3]) with spread:");
console.log(sum(1, ...args));

class Calculator {
    base: number;
    constructor(base: number) {
        this.base = base;
    }
    add(a: number, b: number): number {
        return this.base + a + b;
    }
}

let calc = new Calculator(10);
let calcArgs = [5, 5];
console.log("Testing method call with spread (Calculator.add):");
console.log(calc.add(...calcArgs));
