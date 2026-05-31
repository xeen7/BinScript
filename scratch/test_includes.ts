function main() {
  const email = "john@example.com";
  console.log("email: " + email);
  console.log("email.includes('@'): " + email.includes("@"));
  console.log("email.includes('john'): " + email.includes("john"));
  console.log("email.includes('xyz'): " + email.includes("xyz"));
}
main();
