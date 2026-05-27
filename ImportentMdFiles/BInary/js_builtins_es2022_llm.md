# JavaScript Built-in Objects — ES2022

## Primitive Wrappers
| Object | Description |
|--------|-------------|
| `Object` | Root of all objects; base prototype chain |
| `Function` | Callable object; every function is an instance |
| `Boolean` | Wrapper for `true` / `false` |
| `Number` | Wrapper for 64-bit IEEE 754 floats; numeric constants |
| `BigInt` | Arbitrary-precision integers |
| `String` | Wrapper for UTF-16 character sequences |
| `Symbol` | Unique, non-string property keys |

## Math
| Object | Description |
|--------|-------------|
| `Math` | Namespace of static math functions and constants (`PI`, `sqrt`, `random`, etc.) |

## Date & Time
| Object | Description |
|--------|-------------|
| `Date` | Point in time; milliseconds since Unix epoch |

## Text & Patterns
| Object | Description |
|--------|-------------|
| `RegExp` | Regular expression pattern matcher |

## Indexed Collections
| Object | Description |
|--------|-------------|
| `Array` | Ordered, resizable list of any values |
| `Int8Array` | Typed array of 8-bit signed integers |
| `Uint8Array` | Typed array of 8-bit unsigned integers |
| `Uint8ClampedArray` | Typed array of 8-bit unsigned integers clamped to 0–255 |
| `Int16Array` | Typed array of 16-bit signed integers |
| `Uint16Array` | Typed array of 16-bit unsigned integers |
| `Int32Array` | Typed array of 32-bit signed integers |
| `Uint32Array` | Typed array of 32-bit unsigned integers |
| `Float32Array` | Typed array of 32-bit IEEE floats |
| `Float64Array` | Typed array of 64-bit IEEE floats |
| `BigInt64Array` | Typed array of 64-bit signed BigInt integers |
| `BigUint64Array` | Typed array of 64-bit unsigned BigInt integers |

## Keyed Collections
| Object | Description |
|--------|-------------|
| `Map` | Ordered key-value pairs; any value usable as key |
| `Set` | Ordered collection of unique values |
| `WeakMap` | Key-value map with weakly referenced object keys |
| `WeakSet` | Collection of weakly referenced objects |
| `WeakRef` | Weak reference to an object without preventing GC |
| `FinalizationRegistry` | Registers cleanup callbacks when objects are garbage-collected |

## Structured / Binary Data
| Object | Description |
|--------|-------------|
| `ArrayBuffer` | Fixed-length raw binary memory buffer |
| `SharedArrayBuffer` | Raw binary buffer shareable across threads |
| `DataView` | Low-level byte-level reader/writer over an `ArrayBuffer` |
| `Atomics` | Atomic read-modify-write operations on `SharedArrayBuffer` |

## Serialization
| Object | Description |
|--------|-------------|
| `JSON` | Serialize to / parse from JSON text |

## Control & Asynchrony
| Object | Description |
|--------|-------------|
| `Promise` | Future value; eventual success or failure of an async operation |
| `GeneratorFunction` | Constructor for `function*` generators |
| `Generator` | Iterator returned by a generator function |
| `AsyncFunction` | Constructor for `async function` |
| `AsyncGeneratorFunction` | Constructor for `async function*` |
| `AsyncGenerator` | Async iterator returned by an async generator function |

## Reflection & Metaprogramming
| Object | Description |
|--------|-------------|
| `Proxy` | Intercepts and redefines fundamental operations on an object |
| `Reflect` | Static methods mirroring `Proxy` traps; low-level object operations |

## Errors
| Object | Description |
|--------|-------------|
| `Error` | Base error type |
| `EvalError` | Error related to `eval()` |
| `RangeError` | Value outside its valid range |
| `ReferenceError` | Accessing an undefined variable |
| `SyntaxError` | Unparseable code |
| `TypeError` | Wrong type for an operation |
| `URIError` | Malformed URI in `encodeURI` / `decodeURI` |
| `AggregateError` | Multiple errors bundled together (used by `Promise.any`) |

## Internationalization (`Intl`)
| Object | Description |
|--------|-------------|
| `Intl` | Namespace for all internationalization APIs |
| `Intl.Collator` | Locale-sensitive string comparison |
| `Intl.DateTimeFormat` | Locale-aware date and time formatting |
| `Intl.NumberFormat` | Locale-aware number and currency formatting |
| `Intl.ListFormat` | Formats arrays into locale-aware lists |
| `Intl.PluralRules` | Determines plural category for a number |
| `Intl.RelativeTimeFormat` | Formats relative time strings ("3 days ago") |
| `Intl.Segmenter` | Segments text into graphemes, words, or sentences |
| `Intl.DisplayNames` | Translates language, region, and script codes to display names |
| `Intl.Locale` | Represents and manipulates a BCP 47 locale identifier |

## Global Functions & Values
| Name | Description |
|------|-------------|
| `globalThis` | The global object, uniform across environments |
| `undefined` | The `undefined` primitive value |
| `NaN` | IEEE 754 Not-a-Number value |
| `Infinity` | IEEE 754 positive infinity |
| `eval` | Parses and executes a string as JavaScript code |
| `isFinite` | Returns `true` if value is a finite number |
| `isNaN` | Returns `true` if value is `NaN` |
| `parseFloat` | Parses a string into a floating-point number |
| `parseInt` | Parses a string into an integer with optional radix |
| `encodeURI` | Encodes a full URI, preserving URI-structural characters |
| `decodeURI` | Decodes a full encoded URI |
| `encodeURIComponent` | Encodes a URI component, escaping all special characters |
| `decodeURIComponent` | Decodes an encoded URI component |

---

## Ways to Create Objects

| Syntax | Description |
|--------|-------------|
| `{}` | Object literal — inline key-value definition |
| `new Object()` | Explicit `Object` constructor call |
| `new ClassName()` | Constructor function or `class` instantiation |
| `Object.create(proto)` | New object with specified prototype; pass `null` for no prototype |
| `Object.assign({}, src)` | Shallow-copies own enumerable properties from sources into a new object |
| `{ ...obj }` | Spread — shallow copy / merge one or more objects |
| `Object.fromEntries(pairs)` | Builds an object from an iterable of `[key, value]` pairs or a `Map` |
| `JSON.parse(str)` | Deserializes a JSON string into a plain object |
| `structuredClone(obj)` | Deep clone of an object, including nested structures |
| Factory function | Plain function that returns `{}` — encapsulates creation logic without `new` |
| IIFE returning `{}` | Immediately invoked function returning an object; enables private closure state |
| `class` + `new` | Class declaration / expression instantiated with `new` |
| `new Proxy(target, handler)` | Object that intercepts operations; wraps an existing target |
