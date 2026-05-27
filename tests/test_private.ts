class Account {
  #balance: number = 0;
  static #count: number = 0;

  constructor(initial: number) {
    this.#balance = initial;
    Account.#count = Account.#count + 1;
  }

  #getFee(): number {
    return 5;
  }

  withdraw(amount: number): number {
    this.#balance = this.#balance - amount - this.#getFee();
    return this.#balance;
  }

  static getCount(): number {
    return Account.#count;
  }
}

const acc1 = new Account(100);
const acc2 = new Account(200);

console.log(acc1.withdraw(20)); // 100 - 20 - 5 = 75
console.log(Account.getCount()); // 2
