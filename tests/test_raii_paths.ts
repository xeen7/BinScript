// tests/test_raii_paths.ts

// A mock file handle class to test RAII semantics.
class MockFile {
    id: number;
    constructor(id: number) {
        this.id = id;
    }
}

// Emulates opening a file. In BinScript, if DRA identifies `close` as a release function, 
// allocating this object should invoke the RAII layer.
function open(id: number): MockFile {
    let f = new MockFile(id);
    console.log("opened");
    return f;
}

// The release function that should be called automatically.
function close(f: MockFile): void {
    console.log("closed");
}

function test_paths(path: number) {
    let f = open(path);
    
    if (path === 2) {
        return; // Early return path. DRA should insert close(f) here.
    }
    if (path === 3) {
        throw new Error("Oops"); // Throw path. Unwinder should call close(f).
    }
    
    // Normal path
    close(f);
}

function main() {
    console.log("--- normal path ---");
    test_paths(1);
    
    console.log("--- early return ---");
    test_paths(2);
    
    console.log("--- throw path ---");
    try {
        test_paths(3);
    } catch (e) {
        console.log("caught exception");
    }
    
    console.log("--- all paths done ---");
}

main();
