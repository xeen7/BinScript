function main() {
    let u = { name: "Sam" };
    
    let myClosure = (data: any) => {
        let local_name = data.name;
    };
    
    myClosure(u);
}

main();
