async function getVal(x: number) {
    return x;
}

function* promiseGen() {
    yield getVal(10);
    yield getVal(20);
    yield getVal(30);
}

async function main() {
    let gen = promiseGen();
    for await (let x of gen) {
        console.log(x);
    }
    console.log("done");
}

main();
