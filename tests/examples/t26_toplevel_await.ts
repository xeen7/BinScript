async function getMessage() {
    return "delayed message";
}

console.log("Before await");
let result = await getMessage();
console.log("After await:", result);
