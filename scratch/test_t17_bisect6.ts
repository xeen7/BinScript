function assertEqual(actual: any, expected: any, description: string) {
  const result = actual === expected;
  console.log("Assert [" + description + "]: expected " + expected + ", got " + actual + " → " + (result ? "PASS" : "FAIL"));
  if (!result) {
    throw new Error("Assertion failed: " + description);
  }
}

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
}

class BankAccount {
  #balance: number = 0;
  #ownerName: string;
  constructor(ownerName: string, initialDeposit: number) {
    this.#ownerName = ownerName;
    if (initialDeposit > 0) { this.#balance = initialDeposit; }
  }
  get balance() { return this.#balance; }
  get owner() { return this.#ownerName; }
}

function main() {
  console.log("=== START ===");
  runCartTests();
  console.log("Cart done");
  const account = new BankAccount("Alice Smith", 100);
  assertEqual(account.owner, "Alice Smith", "Account owner name");
  console.log("=== DONE ===");
}

main();
