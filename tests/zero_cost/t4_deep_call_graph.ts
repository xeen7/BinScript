function A(obj: any) {
    return B(obj);
}

function B(obj: any) {
    return C(obj);
}

function C(obj: any) {
    return obj.x + 10;
}

function testDeepCall() {
    let sum = 0;
    for (let i = 0; i < 1000; i++) {
        let p = { x: i, y: 0 };
        sum += A(p);
    }
    console.log("Deep Call Graph complete! Final sum: " + sum);
}

testDeepCall();
