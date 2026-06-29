class Node {
    value: number;
    next: any;
    onClick: any;

    constructor(val: number) {
        this.value = val;
    }
}

function leak() {
    let a = new Node(10);
    let b = new Node(20);
    a.next = b;
    b.next = a; // Object cycle WITHOUT CLOSURE

    // No closure, so `circ_destroy` won't drop closure!
    // But wait, the cycle collector should still collect this cycle!
}

for (let i = 0; i < 2; i++) {
    leak();
}
