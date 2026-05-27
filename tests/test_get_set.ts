class Person {
    _name: string;
    _age: number;

    constructor(name: string, age: number) {
        this._name = name;
        this._age = age;
    }

    get name(): string {
        console.log("In name getter");
        return this._name;
    }

    set name(n: string) {
        console.log("In name setter with value: " + n);
        this._name = n;
    }

    get age(): number {
        console.log("In age getter");
        return this._age;
    }

    set age(a: number) {
        console.log("In age setter");
        if (a >= 0) {
            this._age = a;
        } else {
            console.log("Invalid age!");
        }
    }
}

class Employee extends Person {
    role: string;
    constructor(name: string, age: number, role: string) {
        super(name, age);
        this.role = role;
    }
}

console.log("--- Instantiating Person ---");
const p = new Person("Alice", 25);

console.log("Reading name:");
console.log(p.name);

console.log("Writing name:");
p.name = "Bob";
console.log("Reading name after write:");
console.log(p.name);

console.log("Reading age:");
console.log(p.age);
console.log("Writing valid age:");
p.age = 30;
console.log(p.age);

console.log("Writing invalid age:");
p.age = -5;
console.log(p.age);

console.log("--- Instantiating Employee (Inherited Getters/Setters) ---");
const e = new Employee("Charlie", 40, "Engineer");
console.log("Employee name:");
console.log(e.name);
console.log("Setting Employee name:");
e.name = "Diana";
console.log(e.name);
