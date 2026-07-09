function myCustomMap(arr, cb) {
    let result = [];
    result.push(cb(arr[0]));
    return result;
}

function __bs_script_main() {
    let val = { num: 10 };
    // This closure captures val. If myCustomMap is analyzed correctly,
    // the closure doesn't escape and val doesn't escape!
    let out = myCustomMap([1], (x) => x * val.num);
    console.log(out[0]);
}

__bs_script_main();
