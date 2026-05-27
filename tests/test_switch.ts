// Switch Statement Test Suite

function testSwitch(val: any): void {
    console.log("Testing switch with value:");
    console.log(val);
    switch (val) {
        case 1:
            console.log("matched 1");
            break;
        case 2:
            console.log("matched 2 (no break, fallthrough!)");
        case 3:
            console.log("matched 3");
            break;
        default:
            console.log("matched default");
            break;
    }
}

testSwitch(1);
testSwitch(2);
testSwitch(3);
testSwitch(4);

// Test switch inside loop with break and continue
console.log("Testing switch inside loop:");
for (let i = 0; i < 5; i++) {
    switch (i) {
        case 1:
            console.log("case 1, continue loop");
            continue; // Should continue the for loop!
        case 3:
            console.log("case 3, break switch");
            break; // Should break the switch!
        default:
            console.log("default case for:");
            console.log(i);
            break;
    }
    console.log("end of loop iteration");
}
