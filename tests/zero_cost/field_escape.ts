function stealAddress(user: any, arr: any[]) {
    arr.push(user.address);
}

function main() {
    let globalArr: any[] = [];
    let u = { name: "Sam", address: "123 Main St" };
    stealAddress(u, globalArr);
}
main();
