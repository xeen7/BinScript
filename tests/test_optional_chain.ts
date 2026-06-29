function main() {
    const config = {
        theme: "dark",
        settings: {
            notifications: true
        }
    };
    const notifyEnabled = config?.settings?.notifications ?? false;
}
main();
