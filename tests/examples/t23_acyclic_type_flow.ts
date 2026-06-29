// Pure primitives -> acyclic
class Point {
    x: number;
    y: number;
}

// Composes purely primitives -> acyclic
class Line {
    a: Point;
    b: Point;
}

// Self-referential cycle -> cyclic
class Node {
    value: number;
    next: Node;
}

// Reaches self-referential cycle -> cyclic
class List {
    head: Node;
}

// Unannotated or 'any' -> cyclic
class Box {
    val: any;
}

function main() {
    let p = new Point();
    p.x = 10;
    p.y = 20;

    let l = new Line();
    l.a = p;
    l.b = p;

    let n = new Node();
    n.value = 5;
    n.next = n; // Cycle!

    let list = new List();
    list.head = n;

    let b = new Box();
    b.val = b; // Cycle!
    
    console.log(p.x);
}

main();
