async function main() {
    console.log("Reactor Test Start");
    
    const p1 = sleep(100);
    const p2 = sleep(200);
    const p3 = sleep(50);
    
    console.log("Timers scheduled, waiting for first timer...");
    await p3;
    console.log("Timer 3 (50ms) finished!");
    
    await p1;
    console.log("Timer 1 (100ms) finished!");
    
    await p2;
    console.log("Timer 2 (200ms) finished!");
    
    console.log("Reactor Test Complete");
}

await main();
