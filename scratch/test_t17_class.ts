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

console.log("DONE");
