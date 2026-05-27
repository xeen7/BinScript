class Calculator {
  static baseVal = 10;
  static doubleBase: number;

  static {
    // Reference outer class name
    Calculator.baseVal = Calculator.baseVal + 5;
    // Reference this keyword (referring to the Calculator constructor object)
    this.doubleBase = this.baseVal * 2;
  }
}

console.log(Calculator.baseVal); // should print 15
console.log(Calculator.doubleBase); // should print 30
