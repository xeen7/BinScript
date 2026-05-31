function assertEqual(actual: any, expected: any, description: string) {
  const result = actual === expected;
  console.log("Assert [" + description + "]: expected " + expected + ", got " + actual + " -> " + (result ? "PASS" : "FAIL"));
}

function main() {
  const cart: any[] = [
    { id: "p1", name: "Wireless Mouse", price: 25, quantity: 2, inStock: true },
    { id: "p2", name: "USB-C Cable", price: 12.5, quantity: 3, inStock: true },
    { id: "p3", name: "Mechanical Keyboard", price: 99, quantity: 1, inStock: false }, 
    { id: "p4", name: "4K Monitor", price: 299, quantity: 1, inStock: true }
  ];

  console.log("Before filter");
  const activeItems = cart.filter(item => item.inStock);
  console.log("activeItems length: " + activeItems.length);

  console.log("Before map");
  const subtotals = activeItems.map(item => item.price * item.quantity);
  console.log("subtotals length: " + subtotals.length);

  console.log("Before reduce");
  const grandTotal = subtotals.reduce((sum, current) => sum + current, 0);
  console.log("grandTotal: " + grandTotal);
}

main();
