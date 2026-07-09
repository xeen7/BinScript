// Helper function that takes a closure and calls it immediately.
// Since the closure is not returned or stored globally, it does NOT escape.
function call_it(c: () => number) {
    return c();
}

function __bs_script_main() {
    let multiplier = { val: 42 }; // Allocated.
    
    // The closure captures `multiplier`.
    // Because it is only passed to `call_it`, it does NOT escape.
    // The compiler will infer `AllocOwnedClosure` for it.
    // As a result, `multiplier` does NOT need an RcInc instruction!
    let result = call_it(() => {
        return multiplier.val;
    });
    
    console.log(result);
}

__bs_script_main();
