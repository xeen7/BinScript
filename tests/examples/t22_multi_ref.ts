class Node {
    next1: any;
    next2: any;
}
function leak() {
    let a = new Node();
    let b = new Node();
    let c = new Node();
    a.next1 = b;
    c.next1 = b;
    b.next1 = a;
    b.next2 = c;
}
leak();
