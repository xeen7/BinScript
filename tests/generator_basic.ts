function* counter() {
    yield 1;
    yield 2;
    let x = yield 3;
    console.log(x); // should print 42
    return 99;
}

function main() {
    let gen = counter();
    // generator_next is our low-level runtime stub
    console.log(generator_next(gen, 0));  // 1
    console.log(generator_next(gen, 0));  // 2
    console.log(generator_next(gen, 0));  // 3
    console.log(generator_next(gen, 42)); // 42, then 99 (return value)
    console.log(generator_next(gen, 0));  // 0 (exhausted/undefined)
}

main();
