// Test 1: Unique Ownership (Owned)
// The `createPoint` function is a pure factory.
// It returns a fresh allocation that doesn't escape.
// `pt` should be classified as MemoryClass::Owned (0xFFFC) and naturally Dropped.

function assertEqual(name: string, actual: any, expected: any) {
  if (actual !== expected) {
    throw new Error(`Assertion failed: ${name}`);
  }
}

function createPoint(x: number, y: number) {
    return { x: x, y: y };
}

function main() {
    let pt = createPoint(100, 200);
    
    let sum = pt.x + pt.y;
    assertEqual("Owned Factory Sum", sum, 300.0);
}
