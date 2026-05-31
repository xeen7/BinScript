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
    contact: { email: "coder123@example.com", address: { city: "San Francisco" } },
    preferences: { theme: "dark-mode" }
  };
  const user2: any = { username: "guest_user" };
  assertEqual(getUserTheme(user1), "dark-mode", "User1 theme");
  assertEqual(getUserCity(user1), "San Francisco", "User1 city");
  assertEqual(getUserTheme(user2), "system-default", "User2 theme");
  assertEqual(getUserCity(user2), "Unknown City", "User2 city");
  const { username, preferences: { fontSize = 14 } = {} } = user1 as any;
  assertEqual(username, "coder123", "Destructured username");
  assertEqual(fontSize, 14, "Destructured fontSize");
}

function runDummy() {
  console.log("Dummy");
}

function main() {
  console.log("=== START ===");
  runCartTests();
  console.log("Cart done");
  runProfileTests();
  console.log("Profile done");
  runDummy();
  console.log("=== ALL DONE ===");
}

main();
