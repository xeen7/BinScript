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
  emitter.emit("data", "Second Event!");
}
function runTextUtilityTests() {
  const template = "Hi {name}, your package is scheduled for {day}!";
  const params: any = {
    name: "Alex",
    day: "Thursday"
  };
  const keys = Object.keys(params);
}
function main() {
  runEventEmitterTests();
  runTextUtilityTests();
}
main();
