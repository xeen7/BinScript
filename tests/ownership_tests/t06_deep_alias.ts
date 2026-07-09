function __bs_script_main() {
    // Both objects should be AllocOwned because `parent` never escapes.
    let child = { value: 100 };
    let parent = { obj: child };
    
    // Changing the property
    parent.obj.value = 200;
    
    console.log(parent.obj.value); // Prints 200
}

__bs_script_main();
