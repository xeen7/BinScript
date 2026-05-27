async function sleep() {}

async function delay1() {
    console.log(1);
    await sleep();
    await sleep(); // Extra sleep to make it slower
    return 10;
}

async function delay2() {
    console.log(2);
    await sleep();
    return 20;
}

async function main() {
    let p1 = delay1();
    let p2 = delay2();
    let res = await Promise.race_2(p1, p2);
    console.log(res); // 20 should win
}

main();
