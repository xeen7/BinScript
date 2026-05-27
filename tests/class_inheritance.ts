class Animal {
    name: string;
    constructor(name: string) {
        this.name = name;
    }
    speak(): void {
        console.log("animal sound");
    }
}
class Dog extends Animal {
    breed: string;
    constructor(name: string, breed: string) {
        super(name);
        this.breed = breed;
    }
    speak(): void {
        console.log("bark");
    }
}
const d = new Dog("Rex", "German Shepherd");
console.log(d.name);
console.log(d.breed);
d.speak();
