// Test Symbol primitives
const sym1 = Symbol("hello");
const sym2 = Symbol("world");
const sym3 = Symbol();

// typeof
console.log(typeof sym1);    // "symbol"
console.log(typeof sym3);    // "symbol"

// String coercion
console.log(String(sym1));   // "Symbol(hello)"
console.log(String(sym2));   // "Symbol(world)"
console.log(String(sym3));   // "Symbol()"

// Truthiness - symbols are always truthy
if (sym1) {
    console.log("truthy");   // "truthy"
}

// Well-known symbols
console.log(typeof Symbol.iterator);      // "symbol"
console.log(String(Symbol.iterator));     // "Symbol(Symbol.iterator)"
