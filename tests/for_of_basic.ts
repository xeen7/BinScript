function* generateNumbers() {
    yield 10;
    yield 20;
    yield 30;
}

function main() {
    let gen = generateNumbers();
    console.log(gen);
    let sum = 0;
    for (let x of gen) {
        console.log(x);
        sum = sum + x;
    }
    console.log(sum); // 60
}

main();
