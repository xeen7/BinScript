

declare function startMockServer(port: number): any;
declare function stopMockServer(server: any): void;

async function main() {
    console.log("Starting mock local HTTP server...");
    
    const server = startMockServer(8080);
    console.log("Mock server started successfully!");

    try {
        console.log("--- TEST 1: GET Fetch Request ---");
        const url = "http://localhost:8080/get";
        console.log("Fetching: " + url);
        
        const response = await fetch(url);
        
        console.log("Response status: " + response.status);
        console.log("Response ok: " + response.ok);
        console.log("Response statusText: " + response.statusText);

        if (response.status !== 200) {
            throw new Error("Expected status 200, got " + response.status);
        }
        if (response.ok !== true) {
            throw new Error("Expected ok to be true");
        }

        console.log("Checking headers...");
        const contentType = response.headers.get("content-type");
        console.log("Content-Type: " + contentType);
        if (!contentType || contentType.indexOf("application/json") === -1) {
            throw new Error("Expected application/json in Content-Type header");
        }

        console.log("Parsing JSON body...");
        const data = await response.json();
        console.log("Parsed title: " + data.title);
        console.log("Parsed userId: " + data.userId);
        
        if (data.id !== 1) {
            throw new Error("Expected post ID to be 1, got " + data.id);
        }
        if (data.userId !== 1) {
            throw new Error("Expected userId to be 1, got " + data.userId);
        }
        if (data.title !== "TSDroid GET mock") {
            throw new Error("Expected title 'TSDroid GET mock', got " + data.title);
        }

        console.log("--- TEST 2: POST Fetch Request ---");
        const postUrl = "http://localhost:8080/post";
        const postOptions = {
            method: "POST",
            body: '{"title":"TSDroid Network","body":"Hello from TSDroid!","userId":42}',
            headers: {
                "Content-Type": "application/json; charset=UTF-8"
            }
        };
        
        console.log("Posting to: " + postUrl);
        const postResponse = await fetch(postUrl, postOptions);
        console.log("POST status: " + postResponse.status);
        console.log("POST ok: " + postResponse.ok);
        
        if (postResponse.status !== 201) {
            throw new Error("Expected POST status 201, got " + postResponse.status);
        }
        
        const postData = await postResponse.json();
        console.log("POST response id: " + postData.id);
        console.log("POST response title: " + postData.title);
        
        if (postData.title !== "TSDroid Network") {
            throw new Error("Expected title 'TSDroid Network', got " + postData.title);
        }
        
        console.log("--- ALL FETCH API TESTS PASSED SUCCESSFULLY! ---");
    } finally {
        console.log("Stopping mock local HTTP server...");
        
        stopMockServer(server);
        console.log("Mock server stopped successfully!");
    }
}

main();
