// Test 1: Standard Error Objects instantiation and properties
const e1 = new Error("General error occurred");
console.log(e1.name); // "Error"
console.log(e1.message); // "General error occurred"
e1.code = 404; // Add dynamic property
console.log(e1.code); // 404

const e2 = new TypeError("Type mismatch");
console.log(e2.name); // "TypeError"
console.log(e2.message); // "Type mismatch"

const e3 = new RangeError("Index out of bounds");
console.log(e3.name); // "RangeError"
console.log(e3.message); // "Index out of bounds"

const e4 = new ReferenceError("Variable undef is not defined");
console.log(e4.name); // "ReferenceError"
console.log(e4.message); // "Variable undef is not defined"

const e5 = new SyntaxError("Unexpected token");
console.log(e5.name); // "SyntaxError"
console.log(e5.message); // "Unexpected token"

// Test 2: Catching and verifying different error types in a dynamic dispatch loop
function getError(type: string) {
    if (type === "type") {
        return new TypeError("Invalid type");
    } else if (type === "range") {
        return new RangeError("Invalid range");
    } else if (type === "reference") {
        return new ReferenceError("Invalid reference");
    } else if (type === "syntax") {
        return new SyntaxError("Invalid syntax");
    } else {
        return new Error("Invalid error");
    }
}

const types = ["type", "range", "reference", "syntax", "general"];
for (let i = 0; i < 5; i = i + 1) {
    const t = types[i];
    try {
        const err = getError(t);
        throw err;
    } catch (e) {
        console.log("Caught exception:");
        console.log(e.name);
        console.log(e.message);
    }
}
