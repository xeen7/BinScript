class Target {
    value: number;
    constructor(v: number) {
        this.value = v;
    }
}

function test_weakref() {
    let target = new Target(42);
    let weakRef = new WeakRef(target);

    let deref = weakRef.deref();
    if (deref !== undefined) {
        console.log("WeakRef deref successfully got target!");
    } else {
        console.log("WeakRef target is undefined!");
    }

    // keep target alive
    console.log("Target value is:", target.value);
}

test_weakref();
// At this point, the weakRef target should be garbage collected.
