// Static Class Methods and Fields Test Suite

class Calculator {
    static baseVal = 10;
    static doubleBase: any; // starts as undefined
    
    static add(a: number, b: number): number {
        return Calculator.baseVal + a + b;
    }
}

console.log("Initial Calculator.baseVal:");
console.log(Calculator.baseVal); // should be 10

console.log("Calling Calculator.add(5, 5):");
console.log(Calculator.add(5, 5)); // should be 10 + 5 + 5 = 20

// Modifying static field
Calculator.baseVal = 20;
console.log("Updated Calculator.baseVal:");
console.log(Calculator.baseVal); // should be 20

console.log("Calling Calculator.add(5, 5) after update:");
console.log(Calculator.add(5, 5)); // should be 20 + 5 + 5 = 30

// Testing undefined static field
console.log("Initial Calculator.doubleBase:");
console.log(Calculator.doubleBase); // should be undefined

Calculator.doubleBase = 40;
console.log("Updated Calculator.doubleBase:");
console.log(Calculator.doubleBase); // should be 40
