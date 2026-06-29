class Foo {
  name: string;
  constructor(name: string) {
    this.name = name;
  }
}

const f = new Foo("hello");
console.log(f.name);
