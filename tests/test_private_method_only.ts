// Minimal test: private method
class Foo {
  #bar(): number {
    return 42;
  }

  callBar(): number {
    return this.#bar();
  }
}

const f = new Foo();
console.log(f.callBar()); // 42
