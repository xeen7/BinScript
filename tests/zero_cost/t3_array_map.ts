function calculateTotalTime() {
    let jobs = [
        { startTime: 10, endTime: 50 },
        { startTime: 20, endTime: 80 },
        { startTime: 5, endTime: 15 }
    ];

    let total = jobs
        .map((job: any) => job.endTime - job.startTime) 
        .reduce((sum: any, time: any) => sum + time, 0);
        
    console.log("Total job time: " + total);
}

calculateTotalTime();
