class Node {
    value: number;
    next: any;
    onClick: any;

    constructor(val: number) {
        this.value = val;
        this.onClick = () => {
            return this.value;
        };
    }
}

function testCycle() {
    let a = new Node(10);
    let b = new Node(20);
    a.next = b;
    b.next = a; // Object cycle
    
    // a.onClick creates a closure that captures `a` (`this`).
    // This creates a cycle: a -> a.onClick -> a.
}

for (let i = 0; i < 10000; i++) {
    testCycle();
}
