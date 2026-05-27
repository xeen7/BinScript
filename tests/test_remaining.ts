// 1. Test Dynamic Import
async function testDynamicImport() {
    let p = import("./modules");
    console.log(typeof p); // "object" (Promise)
    
    let ns = await p;
    console.log(typeof ns); // "object" (resolved module namespace)
}

// 2. Test with statement
function testWith() {
    // Math is a global object, we simulate a with block
    let obj = { x: 42 };
    with (obj) {
        console.log("Inside with block");
    }
}

// 3. Test TS Decorators (TypeScript/Legacy proposal)
function sealed(constructor: Function) {
    console.log("sealed decorator run");
}

@sealed
class MyDecoratedClass {
    greet() {
        console.log("decorated class greet");
    }
}

async function main() {
    await testDynamicImport();
    testWith();
    
    let instance = new MyDecoratedClass();
    instance.greet();
    
    console.log("all remaining tests completed!");
}

main();
