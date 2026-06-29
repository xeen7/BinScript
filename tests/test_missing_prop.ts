function main() {
  const config = {
    theme: "dark",
    settings: {
      notifications: true
    }
  };

  const missingVal = (config?.settings as any)?.missingProp ?? "default_value";
}
main();
