class PrivateInTest {
    #x = 10;
    
    hasX(obj: any) {
        return #x in obj;
    }
}

let p = new PrivateInTest();
console.log("hasX(p):", p.hasX(p));
console.log("hasX({}):", p.hasX({}));

async function testImport() {
    let m = await import("./some_module.ts", { with: { type: "json" } });
    console.log("imported module!");
}

testImport();
