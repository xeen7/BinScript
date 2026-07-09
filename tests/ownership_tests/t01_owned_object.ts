function pure_math(a: number, b: number) {
    return a + b;
}

function main() {
    // This object doesn't escape and isn't aliased anywhere else.
    // The compiler will correctly infer it as `Owned` and emit a direct `Drop` 
    // instead of reference counting it.
    let obj = { val: 42 };
    
    // Pass primitive values extracted from the object to a pure function.
    let res = pure_math(obj.val, 10);
    console.log(res);
}

main();
