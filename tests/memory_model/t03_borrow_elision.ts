// Test 3: Borrowing (Zero-Cost RC Elision)
// The `parent.child` property read is technically a LoadProp, which generates Shared by default.
// However, because `child` and `a` are only used locally and `parent` stays alive,
// the RcElision optimization pass completely strips their RcInc and RcDec instructions.

function assertEqual(name: string, actual: any, expected: any) {
  if (actual !== expected) {
    throw new Error(`Assertion failed: ${name}`);
  }
}

function main() {
    let parent = { child: { value: 42 } };
    
    // Borrowing the property
    let child = parent.child;
    
    // Using the borrow
    let a = child.value;
    let b = a + 10;
    
    assertEqual("Borrow Arithmetic", b, 52.0);
    
    // Keeping parent alive ensures it outlives the borrow,
    // allowing the optimizer to safely strip the atomic RC operations.
    let c = parent; 
}
