function main() {
  console.log("=== RUNNING COMPLEX FEATURES TEST ===");

  console.log("1. Testing Loop Control Flow (Break/Continue):");
  let primeSum = 0;
  let count = 0;

  for (let i = 2; i < 20; i++) {
    let isPrime = true;
    for (let j = 2; j < i; j++) {
      if (i % j === 0) {
        isPrime = false;
        break;
      }
    }
    if (!isPrime) {
      continue;
    }
    primeSum += i;
    count++;
  }

  console.log("Sum of primes under 20:");
  console.log(primeSum); 
  console.log("Count of primes:");
  console.log(count); 

  console.log("2. Testing Nested Exceptions & Rethrows:");
  let outerCatchRan = false;
  let innerCatchRan = false;
  let innerFinallyRan = false;
  let outerFinallyRan = false;
  let caughtErrorMsg: any = "";

  try {
    try {
      console.log("  Entering inner try...");
      throw "Inner Error Message";
    } catch (e) {
      innerCatchRan = true;
      console.log("  Inner catch caught exception:");
      console.log(e);
      throw e; 
    } finally {
      innerFinallyRan = true;
      console.log("  Inner finally executed.");
    }
  } catch (err) {
    outerCatchRan = true;
    caughtErrorMsg = err;
    console.log("  Outer catch caught rethrown exception:");
    console.log(err);
  } finally {
    outerFinallyRan = true;
    console.log("  Outer finally executed.");
  }

  console.log("Outer catch ran:");
  console.log(outerCatchRan);
  console.log("Inner catch ran:");
  console.log(innerCatchRan);
  console.log("Inner finally ran:");
  console.log(innerFinallyRan);
  console.log("Outer finally ran:");
  console.log(outerFinallyRan);
  console.log("Caught message is correct:");
  console.log(caughtErrorMsg === "Inner Error Message");

  console.log("3. Testing Stateful Closures:");
  let counter = { value: 0 };
  let makeIncrementer = (incrementBy: number) => {
    return () => {
      counter.value = (counter.value as number) + incrementBy;
      return counter.value;
    };
  };

  let incBy2 = makeIncrementer(2);
  let incBy5 = makeIncrementer(5);

  console.log("Increment by 2 first time:");
  console.log(incBy2()); 
  console.log("Increment by 5 first time:");
  console.log(incBy5()); 
  console.log("Increment by 2 second time:");
  console.log(incBy2()); 
  console.log("Final counter value is 9:");
  console.log(counter.value === 9);

  console.log("=== COMPLEX FEATURES TEST COMPLETED SUCCESSFULLY ===");
}
main();
