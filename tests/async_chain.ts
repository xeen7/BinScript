async function step1(x: number) {
    return x + 1;
}

async function step2(x: number) {
    return x * 2;
}

async function step3(x: number) {
    let a = await step1(x);
    let b = await step2(a);
    return b + 10;
}

async function main() {
    let result = await step3(5);
    console.log(result); // (5 + 1) * 2 + 10 = 22
}

main();
