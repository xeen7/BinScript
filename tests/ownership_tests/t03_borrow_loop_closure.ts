// Helper to execute a closure
function exec_closure(c: () => void) {
    c();
}

function __bs_script_main() {
    let sum = { val: 0 }; // Allocated as Owned
    let max = { val: 5 }; // Allocated as Owned

    // In a loop, we create closures. 
    // Because they are passed to exec_closure and never returned,
    // they do NOT escape. The compiler emits AllocOwnedClosure and 
    // ZERO reference counting operations occur inside the loop!
    for (let i = 0; i < max.val; i++) {
        exec_closure(() => {
            sum.val = sum.val + i;
        });
    }

    console.log(sum.val); // Prints 10 (0+1+2+3+4)
}

__bs_script_main();
