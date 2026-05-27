class Config {
  static _debug: boolean = false;

  static get debug(): boolean {
    return Config._debug;
  }

  static set debug(val: boolean) {
    Config._debug = val;
  }
}

console.log(Config.debug);   // false
Config.debug = true;
console.log(Config.debug);   // true (1)
