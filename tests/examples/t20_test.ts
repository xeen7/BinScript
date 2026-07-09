function main() {
    const errors: any = {};
    errors.name = "John";
    console.log("errors.name = " + errors.name);
    const keys = Object.keys(errors);
    console.log("keys.length = " + keys.length);
    console.log("keys[0] = " + keys[0]);
}
main();
