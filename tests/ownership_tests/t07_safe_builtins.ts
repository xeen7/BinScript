function __bs_script_main() {
    let myObj = { val: 42 };
    
    // Passing myObj to console.log should NOT force it to escape,
    // because console.log is a safe built-in. 
    // myObj should still be AllocOwned.
    console.log(myObj);
}

__bs_script_main();
