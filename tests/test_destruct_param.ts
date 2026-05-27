// Parameter Destructuring Test Suite

// 1. Standard function parameter destructuring
console.log("--- 1. Standard function parameter destructuring ---");
function f({ x, y }: any) {
    console.log(x);
    console.log(y);
}
f({ x: 1, y: 2 }); // should print 1, 2

// 2. Function expression parameter destructuring
console.log("--- 2. Function expression parameter destructuring ---");
const fExpr = function([a, b]: any) {
    console.log(a);
    console.log(b);
};
fExpr([3, 4]); // should print 3, 4

// 3. Arrow function parameter destructuring
console.log("--- 3. Arrow function parameter destructuring ---");
const arrow = ({ p, q }: any) => {
    console.log(p);
    console.log(q);
};
arrow({ p: 5, q: 6 }); // should print 5, 6

// 4. Default parameter pattern values
console.log("--- 4. Default parameter pattern values ---");
function g({ a = 10, b = 20 }: any) {
    console.log(a);
    console.log(b);
}
g({ a: 99 }); // should print 99, 20

// 5. Class method parameter destructuring
console.log("--- 5. Class method parameter destructuring ---");
class Tester {
    speak([first, second]: any) {
        console.log(first);
        console.log(second);
    }
}
const t = new Tester();
t.speak([7, 8]); // should print 7, 8
