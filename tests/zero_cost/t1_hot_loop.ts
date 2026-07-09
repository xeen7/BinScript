function distance(p1: any, p2: any) {
    return Math.sqrt((p2.x - p1.x)**2 + (p2.y - p1.y)**2);
}

function updateSimulation() {
    let d = 0;
    for (let i = 0; i < 1000; i++) {
        let a = { x: i, y: i * 2 };
        let b = { x: i + 1, y: i * 3 };
        d = distance(a, b);
    }
    console.log("Simulation complete! Final distance: " + d);
}

updateSimulation();
