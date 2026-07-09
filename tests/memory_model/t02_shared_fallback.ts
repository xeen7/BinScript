// Test 2: Reference Counting Fallback (Shared)
// The `createAndCachePoint` function stores the object globally.
// This triggers EscapeFact::StoreGlobal, causing the compiler to disqualify it as a factory.
// `p` must safely fall back to MemoryClass::Shared (0xFFF6) with RcInc/RcDec.

function assertEqual(name: string, actual: any, expected: any) {
  if (actual !== expected) {
    throw new Error(`Assertion failed: ${name}`);
  }
}

let globalCache: any = null;

function createAndCachePoint(x: number, y: number) {
    let p = { x: x, y: y };
    globalCache = p; // ESCAPE!
    return p;
}

function main() {
    let p = createAndCachePoint(50, 50);
    
    let sum = p.x + p.y;
    assertEqual("Shared Fallback Sum", sum, 100.0);
    
    // globalCache still points to p!
    assertEqual("Global Cache Valid", globalCache.x, 50.0);
}
