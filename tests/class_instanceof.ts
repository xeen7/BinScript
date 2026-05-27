class Shape {}
class Circle extends Shape {}
class Square extends Shape {}

const c = new Circle();
console.log(c instanceof Circle); // Output: true
console.log(c instanceof Shape);  // Output: true
console.log(c instanceof Square); // Output: false
