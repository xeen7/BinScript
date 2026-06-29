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

function test_loop() {
    console.log("--- loop path ---");
    for (let i = 1; i <= 3; i++) {
        let f = open(i);
        if (i === 2) {
            console.log("break out of loop");
            break; // Should close(f) here for i=2
        }
        close(f);
    }
}

function main() {
    test_loop();
    console.log("--- done ---");
}

main();
