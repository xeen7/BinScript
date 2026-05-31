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
  const hasExpensive = cart.some(item => item.price > 250);
  const allInStock = cart.every(item => item.inStock);
  assertEqual(hasExpensive, true, "Cart contains expensive items (> 250)");
  assertEqual(allInStock, false, "All cart items are in stock");
}

function getUserTheme(user: any): any {
  return user.preferences?.theme ?? "system-default";
}

function getUserCity(user: any): any {
  return user.contact?.address?.city ?? "Unknown City";
}

function runProfileTests() {
  const user1: any = {
    username: "coder123",
    contact: {
      email: "coder123@example.com",
      address: {
        city: "San Francisco"
      }
    },
    preferences: {
      theme: "dark-mode"
    }
  };
  const user2: any = {
    username: "guest_user"
  };
  assertEqual(getUserTheme(user1), "dark-mode", "User1 theme preference");
  assertEqual(getUserCity(user1), "San Francisco", "User1 city address");
  assertEqual(getUserTheme(user2), "system-default", "User2 theme fallback");
  assertEqual(getUserCity(user2), "Unknown City", "User2 city fallback");
  const { username, preferences: { fontSize = 14 } = {} } = user1 as any;
  assertEqual(username, "coder123", "Destructured renamed variable");
  assertEqual(fontSize, 14, "Destructured default fontSize");
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
  console.log("Entering runClassTests");
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
  console.log("Exiting runClassTests");
}

function main() {
  console.log("=== START ===");
  runCartTests();
  console.log("Cart done");
  runProfileTests();
  console.log("Profile done");
  runTemplateTests();
  console.log("Template done");
  runClassTests();
  console.log("=== ALL DONE ===");
}

main();
