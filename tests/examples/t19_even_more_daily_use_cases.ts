function assertEqual(actual: any, expected: any, description: string) {
  const result = actual === expected;
  console.log("Assert [" + description + "]: expected " + expected + ", got " + actual + " → " + (result ? "PASS" : "FAIL"));
  if (!result) {
    throw new Error("Assertion failed: " + description);
  }
}

class EventEmitter {
  #listeners: any = {};

  on(event: string, callback: any) {
    if (!this.#listeners[event]) {
      this.#listeners[event] = [];
    }
    this.#listeners[event].push(callback);
  }

  emit(event: string, arg: any) {
    const callbacks = this.#listeners[event];
    console.log("emit event: " + event + ", callbacks present: " + (callbacks !== undefined && callbacks !== null) + ", count: " + (callbacks ? callbacks.length : 0));
    if (callbacks) {
      for (let i = 0; i < callbacks.length; i++) {
        console.log("callbacks[" + i + "] type: " + typeof callbacks[i]);
        console.log("callbacks[" + i + "] content: " + callbacks[i]);
        callbacks[i](arg);
      }
    }
  }
}

function runEventEmitterTests() {
  const emitter = new EventEmitter();
  const state = {
    receivedData: "",
    callCount: 0
  };

  emitter.on("data", (data: any) => {
    console.log("CALLBACK INVOKED with data: " + data);
    state.receivedData = data;
    state.callCount = state.callCount + 1;
  });

  console.log("Emitting data first time...");
  emitter.emit("data", "Hello Event!");
  console.log("After first emit, state.receivedData: " + state.receivedData);
  assertEqual(state.receivedData, "Hello Event!", "Event data received by listener");
  assertEqual(state.callCount, 1, "Listener called exactly once");

  emitter.emit("data", "Second Event!");
  assertEqual(state.receivedData, "Second Event!", "Event data updated on second emit");
  assertEqual(state.callCount, 2, "Listener call count incremented");
}

function slugify(title: string): string {
  let slug = "";
  for (let i = 0; i < title.length; i++) {
    const char = title.substring(i, i + 1);
    if (char === " ") {
      slug += "-";
    } else {
      
      const code = char.charCodeAt(0);
      if (code >= 65 && code <= 90) {
        slug += String.fromCharCode(code + 32);
      } else {
        slug += char;
      }
    }
  }
  return slug;
}

function replaceTemplate(template: string, params: any): string {
  let result = template;
  const keys = Object.keys(params);
  for (let i = 0; i < keys.length; i++) {
    const key = keys[i];
    const placeholder = "{" + key + "}";
    const value = params[key];

    while (result.includes(placeholder)) {
      const idx = result.indexOf(placeholder);
      result = result.substring(0, idx) + value + result.substring(idx + placeholder.length);
    }
  }
  return result;
}

function runTextUtilityTests() {
  
  const title = "My First Blog Post";
  assertEqual(slugify(title), "my-first-blog-post", "Slugified title string");

  const template = "Hi {name}, your package is scheduled for {day}!";
  const params: any = {
    name: "Alex",
    day: "Thursday"
  };
  const rendered = replaceTemplate(template, params);
  assertEqual(rendered, "Hi Alex, your package is scheduled for Thursday!", "Rendered template string substitution");
}

function sanitizePrices(rawItems: any[]): any[] {
  const cleanItems: any[] = [];
  for (let i = 0; i < rawItems.length; i++) {
    const item = rawItems[i];

    const name = item.name ?? "Unnamed Item";

    let rawPrice = item.price;
    let price = 0.0;
    if (rawPrice !== null && rawPrice !== undefined) {
      if (typeof rawPrice === "number") {
        price = rawPrice;
      } else if (typeof rawPrice === "string") {
        
        price = parseFloat(rawPrice);
      }
    }

    const active = item.active ?? true;
    
    cleanItems.push({
      name: name,
      price: price,
      active: active
    });
  }
  return cleanItems;
}

function parseFloat(val: string): number {
  
  let num = 0.0;
  let hasDot = false;
  let divisor = 1.0;
  
  for (let i = 0; i < val.length; i++) {
    const char = val.substring(i, i + 1);
    if (char === ".") {
      hasDot = true;
      continue;
    }
    const digit = char.charCodeAt(0) - 48;
    if (digit >= 0 && digit <= 9) {
      if (!hasDot) {
        num = num * 10 + digit;
      } else {
        divisor *= 10;
        num = num + (digit / divisor);
      }
    }
  }
  return num;
}

function runSanitizerTests() {
  const rawItems: any[] = [
    { name: "Screwdriver", price: "12.99" },
    { price: 45.0, active: false },
    { name: "Hammer", price: null }
  ];

  const cleaned = sanitizePrices(rawItems);

  assertEqual(cleaned[0].name, "Screwdriver", "Sanitized first item name");
  assertEqual(cleaned[0].price, 12.99, "Parsed first item string price");
  assertEqual(cleaned[0].active, true, "Default active status for first item");

  assertEqual(cleaned[1].name, "Unnamed Item", "Default name fallback for second item");
  assertEqual(cleaned[1].price, 45.0, "Kept numeric price for second item");
  assertEqual(cleaned[1].active, false, "Preserved false active status for second item");

  assertEqual(cleaned[2].name, "Hammer", "Third item name");
  assertEqual(cleaned[2].price, 0.0, "Null price fell back to 0.0");
}

function main() {
  console.log("=== RUNNING EVEN MORE DAILY USE CASES TEST SUITE ===");
  
  runEventEmitterTests();
  runTextUtilityTests();
  runSanitizerTests();

  console.log("=== ALL ADDITIONAL DAILY USE CASES PASSED SUCCESSFULLY ===");
}

main();
