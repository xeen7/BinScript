function main() {
  console.log("=== RUNNING STATEFUL NESTED CLOSURES TEST ===");

  let box = { value: 10 };

  let outer = (x: number) => {
    
    return (y: number) => {
      box.value = (box.value as number) + x + y;
      return box.value;
    };
  };

  let closure1 = outer(5); 
  let closure2 = outer(10); 

  console.log("Closure 1 first run (expect 17):");
  console.log(closure1(2)); 

  console.log("Closure 2 first run (expect 30):");
  console.log(closure2(3)); 

  console.log("Closure 1 second run (expect 40):");
  console.log(closure1(5)); 

  console.log("Final box value is 40:");
  console.log(box.value === 40);
  const num = 5 + 8 + 10;
  console.log(num);
  console.log(typeof num);
  console.log(typeof (5 + 8));
  console.log(typeof [1, 2, 3, 4, 5]);
  console.log("=== STATEFUL NESTED CLOSURES TEST COMPLETED SUCCESSFULLY ===");
}
main();