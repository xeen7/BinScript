class Animal {
    name: string;
    constructor(name: string) {
        this.name = name;
    }
    speak(): number {
        console.log(this.name);
        return 1;
    }
}

class Dog extends Animal {
    breed: string;
    constructor(name: string, breed: string) {
        super(name);
        this.breed = breed;
    }
    bark(): number {
        console.log(this.breed);
        return 2;
    }
}

function test_objects() {
    // 1. Plain object literals
    const p1 = { x: 10, y: 20 };
    console.log(p1.x); // Expected: 10
    console.log(p1.y); // Expected: 20

    // 2. Dynamic property additions
    p1.z = 30;
    console.log(p1.z); // Expected: 30

    // 3. Class instances dynamic property additions
    const d = new Dog("Buddy", "Retriever");
    console.log(d.name);  // Expected: Buddy
    console.log(d.breed); // Expected: Retriever
    
    // Set dynamic properties on class instance
    d.age = 5;
    console.log(d.age); // Expected: 5

    // Speak / Bark methods check
    d.speak();
    d.bark();

    // 4. Deep inheritance / Prototype instanceof
    console.log(d instanceof Dog ? 100 : 0);    // Expected: 100
    console.log(d instanceof Animal ? 200 : 0); // Expected: 200

    // 5. Nested objects and GC verification
    let root = { head: p1 };
    let current = root;
    let i = 0;
    while (i < 20000) {
        let next_node = { value: i };
        // Assign dynamic property
        next_node.link = current;
        current = next_node;
        i = i + 1;
    }

    // Verify that p1 (which is still reachable via root/current chain) is not collected and its dynamic properties are intact
    console.log(root.head.x); // Expected: 10
    console.log(root.head.z); // Expected: 30
}

test_objects();
