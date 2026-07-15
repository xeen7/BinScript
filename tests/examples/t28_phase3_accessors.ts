class Test {
    accessor name = "initial";
    static accessor count = 10;
}

let t = new Test();
console.log("name:", t.name);
t.name = "updated";
console.log("name after set:", t.name);

console.log("static count:", Test.count);
Test.count = 20;
console.log("static count after set:", Test.count);
