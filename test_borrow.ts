function main() {
    let parent = { child: { value: 42 } };
    let child = parent.child;
    
    // Simple local operations
    let a = child.value;
    let b = a + 10;
    
    // Keep parent alive so it acts as the owner!
    let c = parent;
}
