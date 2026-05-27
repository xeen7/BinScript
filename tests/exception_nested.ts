function foo() {
    console.log("foo start");
    throw "nested error";
    console.log("foo end");
}

function bar() {
    try {
        console.log("bar try");
        foo();
    } catch (e) {
        console.log("bar catch");
        console.log(e);
        throw "propagated error";
    } finally {
        console.log("bar finally");
    }
}

try {
    console.log("main try");
    bar();
} catch (e) {
    console.log("main catch");
    console.log(e);
}
