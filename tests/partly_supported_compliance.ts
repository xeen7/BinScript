// =========================================================================
// Comprehensive compliance test for partly-supported ES2021 features
// =========================================================================

// ----- 1. Array Literal Spreads -----

let a: number[] = [1, 2, 3];
let b: number[] = [0, ...a, 4, 5];
console.log(b[0]); // 0
console.log(b[1]); // 1
console.log(b[2]); // 2
console.log(b[3]); // 3
console.log(b[4]); // 4
console.log(b[5]); // 5

let c: number[] = [10, 20];
let d: number[] = [...a, ...c];
console.log(d[0]); // 1
console.log(d[1]); // 2
console.log(d[2]); // 3
console.log(d[3]); // 10
console.log(d[4]); // 20

let empty: number[] = [];
let withEmpty: number[] = [99, ...empty, 100];
console.log(withEmpty[0]); // 99
console.log(withEmpty[1]); // 100

// ----- 2. Object Literal Spreads -----

let base = { x: 1, y: 2 };
let extended = { ...base, z: 3 };
console.log(extended.x); // 1
console.log(extended.y); // 2
console.log(extended.z); // 3

let override_test = { ...base, x: 99 };
console.log(override_test.x); // 99
console.log(override_test.y); // 2

// ----- 3. Object Computed Property Keys -----

let key1: string = "hello";
let computed_obj = { [key1]: 42 };
console.log(computed_obj.hello); // 42

let key2: string = "dynamic";
let computed_obj2 = { static_key: 1, [key2]: 2 };
console.log(computed_obj2.static_key); // 1
console.log(computed_obj2.dynamic);    // 2

// ----- 4. Object Method Properties -----

let calculator = {
  double(n: number) {
    return n * 2;
  },
  add(a: number, b: number) {
    return a + b;
  }
};
console.log(calculator.double(10)); // 20
console.log(calculator.add(3, 7));  // 10

let greeter = {
  greet(n: number) {
    return n * 10;
  }
};
console.log(greeter.greet(4)); // 40

// ----- 5. Super Method Calls -----

class Animal {
  baseValue: number;
  constructor(val: number) {
    this.baseValue = val;
  }
  calculate(x: number): number {
    return this.baseValue + x;
  }
  getValue(): number {
    return this.baseValue;
  }
}

class Dog extends Animal {
  extraValue: number;
  constructor(val: number, extra: number) {
    super(val);
    this.extraValue = extra;
  }
  calculate(x: number): number {
    let baseResult: number = super.calculate(x);
    return baseResult + this.extraValue;
  }
}

let dog = new Dog(10, 5);
console.log(dog.calculate(3));  // 18 (10 + 3 + 5)
console.log(dog.getValue());    // 10
console.log(dog.baseValue);     // 10
console.log(dog.extraValue);    // 5
