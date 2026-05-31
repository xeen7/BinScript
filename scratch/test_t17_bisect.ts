function assertEqual(actual: any, expected: any, description: string) {
  const result = actual === expected;
  console.log("Assert [" + description + "]: expected " + expected + ", got " + actual + " → " + (result ? "PASS" : "FAIL"));
  if (!result) {
    throw new Error("Assertion failed: " + description);
  }
}

// Cart tests (filter/map/reduce/some/every)
function calculateCartTotal(cart: any[]) {
  const activeItems = cart.filter(item => item.inStock);
  const subtotals = activeItems.map(item => item.price * item.quantity);
  const grandTotal = subtotals.reduce((sum, current) => sum + current, 0);
  return grandTotal;
}

function runCartTests() {
  const cart: any[] = [
    { id: "p1", name: "Wireless Mouse", price: 25, quantity: 2, inStock: true },
    { id: "p2", name: "USB-C Cable", price: 12.5, quantity: 3, inStock: true },
    { id: "p3", name: "Mechanical Keyboard", price: 99, quantity: 1, inStock: false }, 
    { id: "p4", name: "4K Monitor", price: 299, quantity: 1, inStock: true }
  ];
  const total = calculateCartTotal(cart);
  assertEqual(total, 386.5, "E-commerce grand total computation");
  const hasExpensive = cart.some(item => item.price > 250);
  const allInStock = cart.every(item => item.inStock);
  assertEqual(hasExpensive, true, "Cart contains expensive items (> 250)");
  assertEqual(allInStock, false, "All cart items are in stock");
}

// Tagged template tests
function formatCurrency(strings: readonly string[], ...values: any[]) {
  let result = strings[0];
  for (let i = 0; i < values.length; i++) {
    const val = values[i];
    const formatted = (i === 2) ? "$" + val : val;
    result += formatted + strings[i + 1];
  }
  return result;
}

function runTemplateTests() {
  console.log("Entering runTemplateTests");
  const item = "Premium Coffee";
  const cost = 5.99;
  const quantity = 3;
  console.log("About to call formatCurrency");
  const invoice = formatCurrency`Receipt: ${quantity}x ${item} at ${cost} each`;
  console.log("Got invoice: " + invoice);
  assertEqual(invoice, "Receipt: 3x Premium Coffee at $5.99 each", "Invoice currency formatting tag");
  console.log("Exiting runTemplateTests");
}

console.log("=== START ===");
runCartTests();
console.log("Cart tests done");
runTemplateTests();
console.log("=== ALL DONE ===");
