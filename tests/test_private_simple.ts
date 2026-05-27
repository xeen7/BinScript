// Simplest private field test
class Box {
  #value: number = 0;

  constructor(v: number) {
    this.#value = v;
  }

  getValue(): number {
    return this.#value;
  }
}

const b = new Box(42);
console.log(b.getValue()); // 42
