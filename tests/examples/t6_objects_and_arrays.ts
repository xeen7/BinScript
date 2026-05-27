function main() {
  console.log("=== RUNNING OBJECTS AND ARRAYS TEST ===");

  const user = {
    name: "Alex",
    details: {
      age: 30,
      skills: ["Rust", "TypeScript", "JVM"],
      address: {
        city: "San Francisco",
        zip: 94105
      }
    },
    active: true
  };

  console.log("User name:");
  console.log(user.name);

  console.log("User age:");
  console.log(user.details.age);

  console.log("First skill:");
  console.log(user.details.skills[0]);

  console.log("City:");
  console.log(user.details.address.city);

  user.details.age = 31;
  console.log("Updated age:");
  console.log(user.details.age);

  user.details.skills[1] = "Java";
  console.log("Updated second skill:");
  console.log(user.details.skills[1]);

  const numbers = [10, 20, 30];
  console.log("Initial array length:");
  console.log(numbers.length);

  numbers.push(40);
  console.log("Length after push:");
  console.log(numbers.length);
  console.log("New element at index 3:");
  console.log(numbers[3]);

  const popped = numbers.pop();
  console.log("Popped element:");
  console.log(popped);
  console.log("Length after pop:");
  console.log(numbers.length);

  const config = {
    theme: "dark",
    settings: {
      notifications: true
    }
  };

  const notifyEnabled = config?.settings?.notifications ?? false;
  console.log("Notifications enabled (coalesced):");
  console.log(notifyEnabled);

  const missingVal = (config?.settings as any)?.missingProp ?? "default_value";
  console.log("Missing property fallback:");
  console.log(missingVal);

  console.log("=== OBJECTS AND ARRAYS TEST COMPLETED SUCCESSFULLY ===");
}
main();
