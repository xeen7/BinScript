function my_func(obj) {
    let x = obj;
    console.log(x.name);
}

function my_escaping_func(obj) {
    globalThis.leak = obj;
}

function main() {
    // This should be an Arena allocation!
    let a = { name: "ArenaObject" };
    my_func(a);

    // This MUST be a Shared allocation!
    let b = { name: "SharedObject" };
    my_escaping_func(b);
}

main();
