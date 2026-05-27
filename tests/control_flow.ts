function testLogical() {
    let a = false || 42;
    console.log(a); // 42

    let b = "hello" && 10;
    console.log(b); // 10

    let c = true || console.log(999); // 999 should not print
    console.log(c); // true

    let d = false && console.log(888); // 888 should not print
    console.log(d); // false
}

function testLoops() {
    let i = 0;
    while (i < 10) {
        i = i + 1;
        if (i == 5) {
            continue;
        }
        if (i == 8) {
            break;
        }
        console.log(i); // 1, 2, 3, 4, 6, 7
    }

    let j = 0;
    do {
        j = j + 1;
        console.log(j); // 1, 2, 3
    } while (j < 3);
}

testLogical();
testLoops();
