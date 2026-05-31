function assertEqual(actual: any, expected: any, description: string) {
  const result = actual === expected;
  console.log("Assert [" + description + "]: expected " + expected + ", got " + actual + " -> " + (result ? "PASS" : "FAIL"));
  if (!result) {
    throw new Error("Assertion failed: " + description);
  }
}

interface CartItem {
  id: string;
  name: string;
  price: number;
  quantity: number;
  inStock: boolean;
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

  const hasExpensive = cart.some(item => item.price > 250);
  const allInStock = cart.every(item => item.inStock);

  assertEqual(hasExpensive, true, "Cart contains expensive items (> 250)");
  assertEqual(allInStock, false, "All cart items are in stock");
}

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
  const item = "Premium Coffee";
  const cost = 5.99;
  const quantity = 3;

  const invoice = formatCurrency`Receipt: ${quantity}x ${item} at ${cost} each`;
  assertEqual(invoice, "Receipt: 3x Premium Coffee at $5.99 each", "Invoice currency formatting tag");
}

class BankAccount {
  #balance: number = 0;
  #ownerName: string;

  constructor(ownerName: string, initialDeposit: number) {
    this.#ownerName = ownerName;
    if (initialDeposit > 0) {
      this.#balance = initialDeposit;
    }
  }

  get balance() {
    return this.#balance;
  }

  get owner() {
    return this.#ownerName;
  }

  deposit(amount: number) {
    if (amount > 0) {
      this.#balance = this.#balance + amount;
      return true;
    }
    return false;
  }

  withdraw(amount: number) {
    if (amount > 0 && this.#balance >= amount) {
      this.#balance = this.#balance - amount;
      return true;
    }
    return false;
  }
}

function runClassTests() {
  const account = new BankAccount("Alice Smith", 100);
  assertEqual(account.owner, "Alice Smith", "Account owner name");
  assertEqual(account.balance, 100, "Initial account balance");

  const depSuccess = account.deposit(50.5);
  assertEqual(depSuccess, true, "Deposit operation successful");
  assertEqual(account.balance, 150.5, "Balance after deposit");

  const withdrawSuccess = account.withdraw(30);
  assertEqual(withdrawSuccess, true, "Withdrawal operation successful");
  assertEqual(account.balance, 120.5, "Balance after withdrawal");

  const overdrawSuccess = account.withdraw(200);
  assertEqual(overdrawSuccess, false, "Overdraft correctly rejected");
  assertEqual(account.balance, 120.5, "Balance remains unaffected by failed overdraft");
}

function main() {
  console.log("=== RUNNING DAILY USE CASES TEST SUITE ===");
  
  console.log("Starting runCartTests...");
  runCartTests();
  console.log("runCartTests PASS!");

  console.log("Starting runTemplateTests...");
  runTemplateTests();
  console.log("runTemplateTests PASS!");

  console.log("Starting runClassTests...");
  runClassTests();
  console.log("runClassTests PASS!");

  console.log("=== ALL DAILY USE CASES PASSED SUCCESSFULLY ===");
}

main();
