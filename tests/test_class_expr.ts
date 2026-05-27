const MyClass = class {
  speak(): string {
    return "hello from class expression";
  }
};

const inst = new MyClass();
console.log(inst.speak()); // "hello from class expression"

// Class expression with extends
class Base {
  val(): number {
    return 42;
  }
}

const SubClass = class extends Base {
  speak(): string {
    return "sub speak";
  }
};

const inst2 = new SubClass();
console.log(inst2.val());   // 42
console.log(inst2.speak()); // "sub speak"
