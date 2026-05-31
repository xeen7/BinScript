function assertEqual(actual: any, expected: any, description: string) {
  const result = actual === expected;
  console.log("Assert [" + description + "]: expected " + expected + ", got " + actual + " → " + (result ? "PASS" : "FAIL"));
  if (!result) {
    throw new Error("Assertion failed: " + description);
  }
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

  get balance() { return this.#balance; }
  get owner() { return this.#ownerName; }

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

function main() {
  console.log("=== START ===");
  const account = new BankAccount("Alice Smith", 100);
  assertEqual(account.owner, "Alice Smith", "Account owner name");
  assertEqual(account.balance, 100, "Initial account balance");
  console.log("=== DONE ===");
}

main();
