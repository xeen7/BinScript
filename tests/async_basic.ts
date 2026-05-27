async function fetchValue(x: number) {
    return x * 2;
}

async function main() {
    let a = await fetchValue(10);
    let b = await fetchValue(20);
    console.log(a); // 20
    console.log(b); // 40
    console.log(a + b); // 60
}

main();
