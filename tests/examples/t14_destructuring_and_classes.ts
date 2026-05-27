function assertEqual(name: string, actual: any, expected: any) {
  if (actual === expected) {
    console.log(`Assert [${name}]: expected ${expected}, got ${actual} → PASS`);
  } else {
    console.error(`Assert [${name}]: expected ${expected}, got ${actual} → FAIL`);
    throw new Error(`Assertion failed: ${name}`);
  }
}

class ConfigManager {
  static #defaultPort = 8080;
  #host = "localhost";
  static debug = false;

  static {
    ConfigManager.debug = true;
  }

  getEndpoint() {
    return this.#host + ":" + ConfigManager.#defaultPort;
  }
}

function testDestructuring() {
  
  const user = {
    name: "Alice",
    details: {
      role: "Admin"
    },
    preferences: [true]
  };

  const {
    name,
    details: { role, age = 30 } = {},
    preferences: [notifications, theme = "dark"] = []
  } = user as any;

  assertEqual("Destructured Name", name, "Alice");
  assertEqual("Destructured Role", role, "Admin");
  assertEqual("Destructured Default Age", age, 30.0);
  assertEqual("Destructured Notification Pref", notifications, true);
  assertEqual("Destructured Default Theme", theme, "dark");

  const colors = ["red"];
  const [primary, secondary = "blue", ...rest] = colors as any;
  assertEqual("Primary Color", primary, "red");
  assertEqual("Default Secondary Color", secondary, "blue");
  assertEqual("Rest Colors Length", rest.length, 0.0);
}

function main() {
  console.log("=== RUNNING ADVANCED DESTRUCTURING & CLASSES TEST SUITE ===");

  const manager = new ConfigManager();
  assertEqual("Class Endpoint Output", manager.getEndpoint(), "localhost:8080");
  assertEqual("Class Static Field Init", ConfigManager.debug, true);

  testDestructuring();

  console.log("=== ALL EXAMPLES COMPLETED SUCCESSFULLY ===");
}

main();
