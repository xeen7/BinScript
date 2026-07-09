let globalArr: any[] = [];

// This function is called from two distinct contexts.
// In one context (call 1), `data` does not escape because `flag` is false.
// In the other context (call 2), `data` escapes because `flag` is true.
function storeIfFlag(data: any, flag: boolean, outArr: any[]) {
    if (flag) {
        outArr.push(data);
    }
}

function main() {
    let outArr: any[] = [];
    
    // Call 1: Safe context. `obj1` does not escape.
    let obj1 = { name: "SafeObj" };
    storeIfFlag(obj1, false, outArr);

    // Call 2: Escaping context. `obj2` escapes.
    let obj2 = { name: "EscapingObj" };
    storeIfFlag(obj2, true, outArr);
}

main();
