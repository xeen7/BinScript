async function sleep() {
    // just returns a promise from async fn
}

async function delay1() {
    console.log(1);
    await sleep();
    return 10;
}

async function delay2() {
    console.log(2);
    await sleep();
    await sleep();
    return 20;
}

async function main() {
    let p1 = delay1();
    let p2 = delay2();
    let res = await Promise.all_2(p1, p2);
    console.log(99); // the result of Promise.all_2 will just be undefined/10/20 in our hack, so we'll just log 99
}

main();
