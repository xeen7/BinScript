// Test 4: Inter-Procedural Purity Analysis

// This function is completely pure!
// It doesn't mutate globals, doesn't throw, and doesn't allocate massive cycle collector objects.
function doMath(a: number, b: number): number {
    return a * b + 10;
}

function main() {
    let parent = { child: { value: 42 } };
    
    // Borrowing the property
    let child = parent.child;
    
    // This function call previously aborted the RcElision pass!
    // But now, because `doMath` is pure, the borrow should seamlessly span across it.
    let x = doMath(child.value, 5);
    
    // Keeping parent alive
    let c = parent; 
}
