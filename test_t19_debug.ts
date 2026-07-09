function assertEqual(actual: any, expected: any, description: string) {
  const result = actual === expected;
  console.log("Assert [" + description + "]: expected " + expected + ", got " + actual + " → " + (result ? "PASS" : "FAIL"));
  if (!result) {
    throw new Error("Assertion failed: " + description);
  }
}

class EventEmitter {
  #listeners: any = {};

  on(event: string, callback: any) {
    if (!this.#listeners[event]) {
      this.#listeners[event] = [];
    }
    this.#listeners[event].push(callback);
  }

  emit(event: string, arg: any) {
    const callbacks = this.#listeners[event];
    if (callbacks) {
      for (let i = 0; i < callbacks.length; i++) {
        callbacks[i](arg);
      }
    }
  }
}

function runEventEmitterTests() {
  const emitter = new EventEmitter();
  const state = {
    receivedData: "",
    callCount: 0
  };

  emitter.on("data", (data: any) => {
    state.receivedData = data;
    state.callCount = state.callCount + 1;
  });

  emitter.emit("data", "Hello Event!");
  assertEqual(state.receivedData, "Hello Event!", "Event data received by listener");
  assertEqual(state.callCount, 1, "Listener called exactly once");

  emitter.emit("data", "Second Event!");
}
runEventEmitterTests();
