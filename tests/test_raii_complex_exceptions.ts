class Resource {
    id: number;
    constructor(id: number) {
        this.id = id;
    }
}

function open(id: number): Resource {
    let r = new Resource(id);
    console.log("opened " + id);
    return r;
}

function close(r: Resource): void {
    console.log("closed " + r.id);
}

function throws_error() {
    let r1 = open(10);
    throw new Error("error from throws_error");
    close(r1);
}

function catches_and_rethrows() {
    let r2 = open(20);
    try {
        let r3 = open(30);
        throws_error();
        close(r3);
    } catch (e) {
        console.log("caught inside catches_and_rethrows");
        let r4 = open(40);
        throw e;
        close(r4);
    }
    close(r2);
}

function complex_control_flow() {
    let r5 = open(50);
    for (let i = 0; i < 3; i = i + 1) {
        let r6 = open(60 + i);
        if (i == 1) {
            catches_and_rethrows();
        }
        close(r6);
    }
    close(r5);
}

function main() {
    console.log("--- starting complex exception test ---");
    let r7 = open(70);
    try {
        complex_control_flow();
    } catch (e) {
        console.log("caught in main");
    }
    close(r7);
    console.log("--- done ---");
}

main();
