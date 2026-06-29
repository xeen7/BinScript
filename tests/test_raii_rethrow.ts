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

function intermediate_func(cond: boolean) {
    let f2 = open(2);
    if (cond) {
        throw new Error("Oops");
    }
    close(f2);
}

function test_rethrow() {
    console.log("--- rethrow path ---");
    let f1 = open(1);
    
    try {
        intermediate_func(true);
    } catch (e) {
        console.log("caught inner");
        throw e; // Rethrow exception
    }
    
    close(f1);
}

function main() {
    try {
        test_rethrow();
    } catch (e) {
        console.log("caught outer");
    }
    console.log("--- done ---");
}

main();
