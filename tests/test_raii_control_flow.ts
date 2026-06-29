function open(id: number): number {
    console.log("opened");
    console.log(id);
    return id;
}

function close(id: number) {
    console.log("closed");
    console.log(id);
}

function test_multiple() {
    let f1 = open(1);
    let f2 = open(2);
    let f3 = open(3);
    
    // Will be closed in reverse order
    close(f3);
    close(f2);
    close(f1);
}

function test_break_continue() {
    for (let i = 0; i < 2; i++) {
        let f = open(10 + i);
        if (i == 0) {
            console.log("continue");
            continue;
        }
        console.log("break");
        break;
        close(f);
    }
}

function main() {
    console.log("--- multiple ---");
    test_multiple();
    console.log("--- break/continue ---");
    test_break_continue();
    console.log("--- done ---");
}

main();
