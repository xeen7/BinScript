function assertEqual(actual: any, expected: any, description: string) {
  const result = actual === expected;
  console.log("Assert [" + description + "]: expected " + expected + ", got " + actual + " → " + (result ? "PASS" : "FAIL"));
  if (!result) {
    throw new Error("Assertion failed: " + description);
  }
}

function formatCurrency(strings: readonly string[], ...values: any[]) {
  console.log("in formatCurrency start");
  let result = strings[0];
  for (let i = 0; i < values.length; i++) {
    const val = values[i];
    const formatted = (i === 2) ? "$" + val : val;
    result += formatted + strings[i + 1];
  }
  console.log("in formatCurrency end");
  return result;
}

function runTemplateTests() {
  console.log("in runTemplateTests start");
  const item = "Premium Coffee";
  const cost = 5.99;
  const quantity = 3;
  console.log("before formatCurrency tag");
  const invoice = formatCurrency`Receipt: ${quantity}x ${item} at ${cost} each`;
  console.log("after formatCurrency tag");
  assertEqual(invoice, "Receipt: 3x Premium Coffee at $5.99 each", "Invoice tag");
  console.log("in runTemplateTests end");
}

function main() {
  console.log("=== START ===");
  runTemplateTests();
  console.log("=== DONE ===");
}

main();
