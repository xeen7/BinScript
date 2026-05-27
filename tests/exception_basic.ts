try {
    console.log("Entering try");
    throw "Oops, an error occurred!";
    console.log("This should not print");
} catch (e) {
    console.log("Entering catch");
    console.log(e);
} finally {
    console.log("Entering finally");
}
console.log("Done");
