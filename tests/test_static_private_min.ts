// Minimal test: static private field only
class Counter {
  static #count: number = 0;

  static increment(): number {
    Counter.#count = Counter.#count + 1;
    return Counter.#count;
  }
}

console.log(Counter.increment()); // 1
console.log(Counter.increment()); // 2
