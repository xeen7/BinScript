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
function runSanitizerTests() {
  const rawItems: any[] = [
    { name: "Screwdriver", price: "12.99" },
    { price: 45.0, active: false },
    { name: "Hammer", price: null }
  ];
  const cleaned = sanitizePrices(rawItems);
}
runSanitizerTests();
