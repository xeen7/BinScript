// Test: private field + private method combination
class Account {
  #balance: number = 0;

  constructor(initial: number) {
    this.#balance = initial;
  }

  #getFee(): number {
    return 5;
  }

  withdraw(amount: number): number {
    this.#balance = this.#balance - amount - this.#getFee();
    return this.#balance;
  }
}

const acc1 = new Account(100);
console.log(acc1.withdraw(20)); // 100 - 20 - 5 = 75
