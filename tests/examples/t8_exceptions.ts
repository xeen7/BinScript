function main() {
  console.log("=== RUNNING EXCEPTIONS TEST ===");

  let finallyExecuted = false;
  let catchExecuted = false;
  let exceptionMessage: any = "";

  try {
    console.log("Entering try block...");
    throw "Standard String Exception";
    console.log("This line should not be executed!");
  } catch (e) {
    catchExecuted = true;
    exceptionMessage = e;
    console.log("Exception caught successfully:");
    console.log(e);
  } finally {
    finallyExecuted = true;
    console.log("Finally block executed.");
  }

  console.log("Verification checks:");
  console.log("Catch block ran:");
  console.log(catchExecuted);
  console.log("Finally block ran:");
  console.log(finallyExecuted);
  console.log("Caught message correct:");
  console.log(exceptionMessage === "Standard String Exception");

  console.log("=== EXCEPTIONS TEST COMPLETED SUCCESSFULLY ===");
}
main();
