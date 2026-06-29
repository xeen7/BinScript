function open(id: number): number {
    console.log("opened " + id);
    return id;
}

function close(id: number) {
    console.log("closed " + id);
}

function d1() {
    let f1 = open(1);
    d2();
    close(f1);
}

function d2() {
    let f2 = open(2);
    d3();
    close(f2);
}

function d3() {
    let f3 = open(3);
    throw new Error("fail in d3");
    close(f3);
}

function main() {
    console.log("--- deep throw ---");
    try {
        d1();
    } catch (e) {
        console.log("caught in main");
    }
    console.log("--- done ---");
}

main();
