function assertEqual(actual: any, expected: any, description: string) {
  const result = actual === expected;
  console.log("Assert [" + description + "]: expected " + expected + ", got " + actual + " -> " + (result ? "PASS" : "FAIL"));
}

function validateForm(formData: any): any {
  const errors: any = {};

  console.log("formData.name: " + formData.name);
  console.log("formData.name.length: " + formData.name.length);
  if (!formData.name || formData.name.length < 3) {
    console.log("FAILED name check");
    errors.name = "Name must be at least 3 characters long";
  }

  console.log("formData.age: " + formData.age);
  if (formData.age === null || formData.age < 18) {
    console.log("FAILED age check");
    errors.age = "You must be at least 18 years old";
  }

  console.log("formData.email: " + formData.email);
  if (!formData.email || !formData.email.includes("@")) {
    console.log("FAILED email check");
    errors.email = "Invalid email format";
  }

  console.log("errors object keys count: " + Object.keys(errors).length);
  return {
    valid: Object.keys(errors).length === 0,
    errors: errors
  };
}

function main() {
  const validData: any = {
    name: "John Doe",
    age: 25,
    email: "john@example.com"
  };

  const validResult = validateForm(validData);
  console.log("validResult.valid: " + validResult.valid);
}

main();
