function assertEqual(actual: any, expected: any, description: string) {
  const result = actual === expected;
  console.log("Assert [" + description + "]: expected " + expected + ", got " + actual + " → " + (result ? "PASS" : "FAIL"));
  if (!result) {
    throw new Error("Assertion failed: " + description);
  }
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
  console.log("Entering runTemplateTests");
  const item = "Premium Coffee";
  const cost = 5.99;
  const quantity = 3;
  const invoice = formatCurrency`Receipt: ${quantity}x ${item} at ${cost} each`;
  assertEqual(invoice, "Receipt: 3x Premium Coffee at $5.99 each", "Invoice currency formatting tag");
}

console.log("=== START ===");
runProfileTests();
console.log("Profile tests done");
runTemplateTests();
console.log("=== ALL DONE ===");
