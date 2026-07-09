function read_obj(obj: any) {
    let x = obj.val;
    return x;
}

function __bs_script_main() {
    let myObj = { val: 42 };
    
    // myObj is passed to read_obj. 
    // read_obj only reads `val` and returns it. It does not store `obj`.
    // Therefore, `obj` should NOT escape read_obj!
    // And `myObj` should remain `AllocOwned`!
    let res = read_obj(myObj);
    console.log(res);
}

__bs_script_main();
