function assertEqual(actual: any, expected: any, description: string) {
  const result = actual === expected;
  console.log("Assert [" + description + "]: expected " + expected + ", got " + actual + " -> " + (result ? "PASS" : "FAIL"));
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

  console.log("Before destructuring in runProfileTests");
  const { username, preferences: { fontSize = 14 } = {} } = user1 as any;
  console.log("After destructuring in runProfileTests");
  assertEqual(username, "coder123", "Destructured renamed variable");
  assertEqual(fontSize, 14, "Destructured default fontSize");
}

function main() {
  console.log("runProfileTests start");
  runProfileTests();
  console.log("runProfileTests end");
}

main();
