function create_multipliers(max: number) {
    let result = [];
    let state = { base: 10 }; // Allocated as Shared

    for (let i = 0; i < max; i++) {
        // This closure is pushed into an array. 
        // Because the array is returned, the closure escapes.
        // Therefore, the compiler is forced to use AllocSharedClosure
        // and emits an RcInc on `state` during creation to keep it alive!
        let closure = () => {
            return i * state.base;
        };
        
        result.push(closure);
    }

    return result;
}

function __bs_script_main() {
    let funcs = create_multipliers(3);
    console.log(funcs[0]()); // 0
    console.log(funcs[1]()); // 10
    console.log(funcs[2]()); // 20
}

__bs_script_main();
