function* inner() {
    yield 2;
    yield 3;
    return 42;
}

function* outer() {
    yield 1;
    let res = yield* inner();
    yield res;
    yield 4;
}

function main() {
    let gen = outer();
    for (let x of gen) {
        console.log(x);
    }
}

main();
