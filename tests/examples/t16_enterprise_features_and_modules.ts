

function assertEqual(name: string, expected: any, actual: any) {
    if (expected === actual) {
        console.log("Assert [" + name + "]: expected " + expected + ", got " + actual + " -> PASS");
    } else {
        console.log("Assert [" + name + "]: expected " + expected + ", got " + actual + " -> FAIL");
        throw new Error("Assertion failed: " + name);
    }
}

function createMultiplier(factor: number) {
    let offset = 10;
    return (x: number) => {
        let localMultiplier = factor * 2;
        return (x * localMultiplier) + offset;
    };
}

class BaseService {
    static serviceVersion = "v2.5.0";
    serviceName: string;

    constructor(name: string) {
        this.serviceName = name;
    }

    getStatus(): string {
        return "Base status: " + this.serviceName;
    }
}

class EnterpriseAuthService extends BaseService {
    private token: string;
    private maxRetries: number;

    constructor(name: string, defaultToken: string) {
        super(name);
        this.token = defaultToken;
        this.maxRetries = 3;
    }

    get authToken(): string {
        return "Bearer " + this.token;
    }

    set authToken(newToken: string) {
        this.token = newToken;
    }

    getStatus(): string {
        return "Enterprise " + super.getStatus() + " (" + this.authToken + ")";
    }

    executeAction(callback: any): string {
        return callback(this.token);
    }
}

function sqlQuery(strings: readonly string[], ...values: any[]) {
    let result: any = strings[0];
    for (let i = 0; i < values.length; i++) {
        result += values[i] + strings[i + 1];
    }
    return result;
}

async function* dataStreamGenerator() {
    yield await Promise.resolve("Packet#1");
    yield await Promise.resolve("Packet#2");
    yield await Promise.resolve("Packet#3");
}

async function main() {
    console.log("=== RUNNING ENTERPRISE ARCHITECTURE TEST SUITE ===");

    let triplerWithOffset = createMultiplier(3); 
    assertEqual("Curried multi-scope closure 1", 40, triplerWithOffset(5)); 
    assertEqual("Curried multi-scope closure 2", 70, triplerWithOffset(10)); 

    assertEqual("Static Class Property", "v2.5.0", BaseService.serviceVersion);
    let authService = new EnterpriseAuthService("OAuthService", "xyz789");
    assertEqual("Inherited and overridden getStatus", "Enterprise Base status: OAuthService (Bearer xyz789)", authService.getStatus());

    authService.authToken = "abc123";
    assertEqual("Getter after setter update", "Bearer abc123", authService.authToken);

    let actionResult = authService.executeAction((t: string) => "Action authenticated with " + t);
    assertEqual("Class method HOF execution", "Action authenticated with abc123", actionResult);

    let configuration: any = {
        app: {
            port: 8080,
            host: "prod.cluster.local"
        },
        flags: [true, false, true],
        meta: {
            owner: "Admin"
        }
    };

    let { app: { port, host }, flags: [flagA, flagB, flagC], meta: { owner, timeout = 5000 } } = configuration;
    assertEqual("Multi-level destructuring port", 8080, port);
    assertEqual("Multi-level destructuring host", "prod.cluster.local", host);
    assertEqual("Multi-level destructuring flagB", false, flagB);
    assertEqual("Multi-level destructuring default timeout", 5000, timeout);

    let tableName = "transactions";
    let minAmount = 500;
    let queryStr = sqlQuery`SELECT * FROM ${tableName} WHERE amount >= ${minAmount} ORDER BY date DESC`;
    assertEqual("Tagged template SQL builder", "SELECT * FROM transactions WHERE amount >= 500 ORDER BY date DESC", queryStr);

    let catchCount = 0;
    let finallyCount = 0;
    try {
        try {
            throw new Error("Critical system fault");
        } catch (innerErr: any) {
            catchCount += 1;
            if (innerErr.message === "Critical system fault") {
                throw innerErr; 
            }
        } finally {
            finallyCount += 1;
        }
    } catch (outerErr: any) {
        catchCount += 1;
        assertEqual("Outer error message verify", "Critical system fault", outerErr.message);
    } finally {
        finallyCount += 1;
    }
    assertEqual("Nested Try-Catch catch count", 2, catchCount);
    assertEqual("Nested Try-Catch finally count", 2, finallyCount);

    let streamedPackets: any[] = [];
    for await (const pkt of dataStreamGenerator()) {
        streamedPackets.push(pkt);
    }
    assertEqual("Async generator packet stream length", 3, streamedPackets.length);
    assertEqual("Async generator packet stream [1]", "Packet#2", streamedPackets[1]);

    console.log("=== ALL ENTERPRISE TESTS COMPLETED SUCCESSFULLY ===");
}

main();
