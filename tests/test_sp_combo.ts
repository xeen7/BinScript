// Test: static private field + instance field, no private method
class Account {
  #balance: number = 0;
  static #count: number = 0;

  constructor(initial: number) {
    this.#balance = initial;
    Account.#count = Account.#count + 1;
  }

  getBalance(): number {
    return this.#balance;
  }

  static getCount(): number {
    return Account.#count;
  }
}

const acc1 = new Account(100);
const acc2 = new Account(200);

console.log(acc1.getBalance()); // 100
console.log(acc2.getBalance()); // 200
console.log(Account.getCount()); // 2
