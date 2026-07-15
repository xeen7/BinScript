class Test {
    static x = 42;
    static getX() { return this.x; }
}
console.log(Test.getX());
