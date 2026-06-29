class MockFile {
    id: number;
    constructor(id: number) {
        this.id = id;
    }
}

function open(id: number): MockFile {
    let f = new MockFile(id);
    console.log("opened " + id);
    return f;
}

function close(f: MockFile): void {
    console.log("closed");
}

function test_nested_return(cond: boolean) {
    console.log("--- nested return ---");
    let f1 = open(1);
    let f2 = open(2);
    
    // Both f2 and f1 should be closed in reverse order
    if (cond) {
        return; 
    }
    
    close(f2);
    close(f1);
}

function test_nested_throw(cond: boolean) {
    console.log("--- nested throw ---");
    let f1 = open(1);
    let f2 = open(2);
    
    if (cond) {
        throw new Error("Oops"); // Both should be closed
    }
    
    close(f2);
    close(f1);
}

function test_nested_normal() {
    console.log("--- nested normal ---");
    let f1 = open(1);
    let f2 = open(2);
    
    close(f2);
    close(f1);
}

function main() {
    test_nested_normal();
    test_nested_return(true);
    try {
        test_nested_throw(true);
    } catch (e) {
        console.log("caught");
    }
    console.log("--- done ---");
}

main();
