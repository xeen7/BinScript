# TypeScript / ECMAScript 2021 — Complete Expressions & Statements Reference

> Intended for LLM consumption. Every expression and statement in the language is listed with a concise description, syntax, and a minimal example. Organized by category.

---

## How to Read This Document

- **Expression** — a piece of code that *produces a value* and can appear on the right-hand side of an assignment or inside another expression.
- **Statement** — a piece of code that *performs an action* and does not itself produce a value (though it may contain expressions).
- **TypeScript-only** items are marked `[TS]`.
- **ECMAScript version** of introduction is noted where relevant.

---

## Part 1 — Expressions

---

### 1.1 Literal Expressions

Literals are the most primitive expressions — they directly represent a fixed value.

| Name | Syntax | Description |
|---|---|---|
| Numeric literal | `42`, `3.14`, `0xFF`, `0b1010`, `0o17`, `1_000_000` | Integer or floating-point number. Supports hex (`0x`), binary (`0b`), octal (`0o`), and numeric separators (`_`). |
| BigInt literal | `42n`, `0xFFn` | Arbitrary-precision integer. Suffix `n`. Cannot mix with `number` without explicit conversion. |
| String literal | `"hello"`, `'world'` | UTF-16 string. Single or double quotes are equivalent. |
| Template literal | `` `Hello ${name}` `` | String with embedded expressions (`${}`). Supports multi-line. |
| Tagged template literal | `` tag`Hello ${name}` `` | Template literal passed to a tag function which receives the string parts and interpolated values as arguments. |
| Boolean literal | `true`, `false` | The two boolean values. |
| Null literal | `null` | The intentional absence of any value. Type is `null`. |
| Undefined | `undefined` | The global `undefined` value. Not a keyword; a property of the global object. |
| RegExp literal | `/pattern/flags` | A regular expression object. Flags: `g`, `i`, `m`, `s`, `u`, `y`, `d`. |
| Array literal | `[1, 2, 3]`, `[a, ...b]` | Creates an array. May contain spread elements (`...`). |
| Object literal | `{ a: 1, b: 2 }` | Creates a plain object with key-value pairs. Supports shorthand, computed keys, methods, and spread. |

---

### 1.2 Identifier & Access Expressions

| Name | Syntax | Description |
|---|---|---|
| Identifier | `x`, `myVar` | A reference to a variable, function, class, or parameter in scope. |
| `this` | `this` | Refers to the current execution context object. Value depends on how the function was called. |
| `super` | `super.method()`, `super(args)` | Refers to the parent class. Used to call parent constructor (`super()`) or parent methods (`super.method()`). |
| Member access | `obj.property` | Reads a named property of an object using dot notation. |
| Computed member access | `obj[expr]` | Reads a property whose name is the result of evaluating `expr`. |
| Optional chaining | `obj?.property`, `obj?.[key]`, `fn?.()` | Accesses a property or calls a function only if the left-hand side is not `null` or `undefined`; short-circuits to `undefined` otherwise. (ES2020) |
| `new.target` | `new.target` | Inside a constructor, refers to the constructor that was directly invoked with `new`. `undefined` if called without `new`. |
| `import.meta` | `import.meta.url` | An object exposing context-specific metadata about the current module (e.g., `url`). (ES2020) |

---

### 1.3 Function & Call Expressions

| Name | Syntax | Description |
|---|---|---|
| Function expression | `function(a, b) { return a + b; }` | Creates a function value. May be named or anonymous. |
| Arrow function expression | `(a, b) => a + b`, `x => x * 2` | Concise function syntax. Does not bind its own `this`, `arguments`, or `super`. |
| Async function expression | `async function() { await fetch(url); }` | Function that returns a `Promise` and may use `await` inside. |
| Async arrow function | `async (x) => await fetch(x)` | Arrow function variant of async. |
| Generator function expression | `function* gen() { yield 1; }` | Function that returns an iterator. Execution is paused at each `yield`. |
| Async generator expression | `async function* gen() { yield await fetch(url); }` | Combines async and generator. Returns an async iterator. |
| Call expression | `fn(a, b)` | Calls a function with the given arguments. |
| `new` expression | `new Dog("Rex")` | Calls a constructor function to create a new object instance. |
| `await` expression | `await promise` | Suspends an async function until a `Promise` resolves. Returns the resolved value. Only valid inside `async` functions. |
| `yield` expression | `yield value` | Pauses a generator and sends `value` to the iterator consumer. Returns the value passed to `.next()`. |
| `yield*` expression | `yield* iterable` | Delegates to another iterable/generator, forwarding all values. |

---

### 1.4 Operator Expressions

#### Arithmetic

| Name | Syntax | Description |
|---|---|---|
| Addition / string concat | `a + b` | Adds two numbers or concatenates if either operand is a string. |
| Subtraction | `a - b` | Numeric subtraction. |
| Multiplication | `a * b` | Numeric multiplication. |
| Division | `a / b` | Numeric division. Returns `Infinity` for division by zero. |
| Remainder | `a % b` | Remainder after integer division. Sign matches dividend. |
| Exponentiation | `a ** b` | `a` raised to the power `b`. (ES2016) |
| Unary plus | `+a` | Converts `a` to a number. |
| Unary negation | `-a` | Negates `a`. |
| Increment (prefix) | `++a` | Increments `a` by 1; evaluates to the *new* value. |
| Increment (postfix) | `a++` | Increments `a` by 1; evaluates to the *old* value. |
| Decrement (prefix) | `--a` | Decrements `a` by 1; evaluates to the *new* value. |
| Decrement (postfix) | `a--` | Decrements `a` by 1; evaluates to the *old* value. |

#### Bitwise

| Name | Syntax | Description |
|---|---|---|
| Bitwise AND | `a & b` | Performs bitwise AND on 32-bit integers. |
| Bitwise OR | `a \| b` | Performs bitwise OR on 32-bit integers. |
| Bitwise XOR | `a ^ b` | Performs bitwise XOR on 32-bit integers. |
| Bitwise NOT | `~a` | Inverts all bits; equivalent to `-(a + 1)`. |
| Left shift | `a << b` | Shifts bits of `a` left by `b` positions; fills with zeros. |
| Signed right shift | `a >> b` | Shifts bits right; fills with sign bit. |
| Unsigned right shift | `a >>> b` | Shifts bits right; fills with zeros regardless of sign. |

#### Logical

| Name | Syntax | Description |
|---|---|---|
| Logical AND | `a && b` | Returns `a` if falsy, otherwise returns `b`. Short-circuits. |
| Logical OR | `a \|\| b` | Returns `a` if truthy, otherwise returns `b`. Short-circuits. |
| Logical NOT | `!a` | Returns `true` if `a` is falsy, `false` otherwise. |
| Nullish coalescing | `a ?? b` | Returns `a` if it is not `null`/`undefined`, otherwise returns `b`. Does not treat `0` or `""` as falsy. (ES2020) |

#### Comparison & Equality

| Name | Syntax | Description |
|---|---|---|
| Loose equality | `a == b` | Equal after type coercion. Avoid in TypeScript. |
| Loose inequality | `a != b` | Not equal after type coercion. |
| Strict equality | `a === b` | Equal without type coercion. Preferred. |
| Strict inequality | `a !== b` | Not equal without type coercion. |
| Less than | `a < b` | True if `a` is numerically/lexicographically less than `b`. |
| Greater than | `a > b` | True if `a` is greater than `b`. |
| Less than or equal | `a <= b` | True if `a` ≤ `b`. |
| Greater than or equal | `a >= b` | True if `a` ≥ `b`. |
| `in` | `"key" in obj` | True if the named property exists on the object or its prototype chain. |
| `instanceof` | `obj instanceof Class` | True if `Class.prototype` exists anywhere in `obj`'s prototype chain. |

#### Assignment

| Name | Syntax | Description |
|---|---|---|
| Assignment | `a = b` | Assigns value of `b` to `a`. Returns `b`. |
| Addition assignment | `a += b` | Shorthand for `a = a + b`. |
| Subtraction assignment | `a -= b` | Shorthand for `a = a - b`. |
| Multiplication assignment | `a *= b` | Shorthand for `a = a * b`. |
| Division assignment | `a /= b` | Shorthand for `a = a / b`. |
| Remainder assignment | `a %= b` | Shorthand for `a = a % b`. |
| Exponentiation assignment | `a **= b` | Shorthand for `a = a ** b`. |
| Bitwise AND assignment | `a &= b` | Shorthand for `a = a & b`. |
| Bitwise OR assignment | `a \|= b` | Shorthand for `a = a \| b`. |
| Bitwise XOR assignment | `a ^= b` | Shorthand for `a = a ^ b`. |
| Left shift assignment | `a <<= b` | Shorthand for `a = a << b`. |
| Right shift assignment | `a >>= b` | Shorthand for `a = a >> b`. |
| Unsigned right shift assignment | `a >>>= b` | Shorthand for `a = a >>> b`. |
| Logical AND assignment | `a &&= b` | Assigns `b` to `a` only if `a` is truthy. Shorthand for `a && (a = b)`. (ES2021) |
| Logical OR assignment | `a \|\|= b` | Assigns `b` to `a` only if `a` is falsy. Shorthand for `a \|\| (a = b)`. (ES2021) |
| Nullish assignment | `a ??= b` | Assigns `b` to `a` only if `a` is `null` or `undefined`. (ES2021) |
| Destructuring assignment | `[a, b] = arr`, `{ x, y } = obj` | Extracts values from arrays or objects into variables. |

#### Other Operators

| Name | Syntax | Description |
|---|---|---|
| Conditional (ternary) | `cond ? a : b` | Returns `a` if `cond` is truthy, `b` otherwise. |
| Comma | `a, b` | Evaluates both operands left-to-right; returns the value of `b`. Used in for-loop headers. |
| `typeof` | `typeof x` | Returns a string describing the type: `"number"`, `"string"`, `"boolean"`, `"object"`, `"function"`, `"undefined"`, `"symbol"`, `"bigint"`. |
| `void` | `void expr` | Evaluates `expr` for side effects and returns `undefined`. |
| `delete` | `delete obj.key` | Removes a property from an object. Returns `true` on success. |
| Spread (in call/array/object) | `fn(...args)`, `[...arr]`, `{...obj}` | Expands an iterable or object into individual elements or properties. |
| Grouping | `(expr)` | Parentheses for explicit precedence control. Evaluates to the inner expression. |
| Sequence | `(a++, b++, c)` | Multiple expressions separated by commas inside parentheses; returns last. |

---

### 1.5 Destructuring Expressions

| Name | Syntax | Description |
|---|---|---|
| Array destructuring | `const [a, b, c] = arr` | Binds array elements to variables by position. |
| Array rest element | `const [first, ...rest] = arr` | Captures remaining elements into an array. |
| Array skip element | `const [, second] = arr` | Skips an element by leaving a blank slot. |
| Object destructuring | `const { x, y } = obj` | Binds object properties to variables by name. |
| Object rename | `const { x: newName } = obj` | Binds property `x` to a variable named `newName`. |
| Object rest | `const { a, ...rest } = obj` | Captures remaining own properties into a new object. |
| Default value (destruct.) | `const { x = 10 } = obj` | Uses default value if the property is `undefined`. |
| Nested destructuring | `const { a: { b } } = obj` | Destructures nested objects. |
| Parameter destructuring | `function f({ x, y }: Point)` | Destructures directly in a function parameter. |

---

### 1.6 Type Expressions (TypeScript Only) `[TS]`

These expressions exist only in TypeScript and are erased at compile time.

| Name | Syntax | Description |
|---|---|---|
| Type assertion (angle) | `<string>value` | Tells the compiler to treat `value` as `string`. Not allowed in TSX. |
| Type assertion (`as`) | `value as string` | Same as angle-bracket assertion; preferred in TSX. |
| Non-null assertion | `value!` | Asserts that `value` is not `null` or `undefined`. Removes `null`/`undefined` from the type. |
| Satisfies operator | `expr satisfies Type` | Checks that `expr` satisfies a type without changing the inferred type of the expression. (TS 4.9) |
| `const` assertion | `[1, 2] as const` | Infers the narrowest possible literal type; makes arrays/objects `readonly`. |

---

### 1.7 Class Expressions

| Name | Syntax | Description |
|---|---|---|
| Class expression | `const Foo = class { ... }` | Creates a class as a value. May be named or anonymous. |
| Class expression with extends | `const Bar = class extends Foo { ... }` | Anonymous class expression that extends another class. |

---

### 1.8 Dynamic Import Expression

| Name | Syntax | Description |
|---|---|---|
| Dynamic import | `import("./module")` | Loads a module asynchronously. Returns a `Promise` resolving to the module's namespace object. (ES2020) |

---

## Part 2 — Statements

---

### 2.1 Declaration Statements

| Name | Syntax | Description |
|---|---|---|
| `var` declaration | `var x = 5;` | Declares a function-scoped (or globally-scoped) variable. Hoisted to the top of its scope. Avoid in modern code. |
| `let` declaration | `let x = 5;` | Declares a block-scoped variable. Not accessible before its declaration (temporal dead zone). |
| `const` declaration | `const x = 5;` | Declares a block-scoped variable that cannot be reassigned. The binding is constant; object contents may still change. |
| Function declaration | `function foo(a, b) { return a + b; }` | Declares a named function in the current scope. Hoisted entirely (body + name). |
| Async function declaration | `async function foo() { }` | Declares an async function. Returns a `Promise`. |
| Generator declaration | `function* foo() { yield 1; }` | Declares a generator function. Returns an iterator. |
| Async generator declaration | `async function* foo() { yield await x; }` | Declares an async generator. Returns an async iterator. |
| Class declaration | `class Foo extends Bar { ... }` | Declares a named class. Not hoisted (temporal dead zone applies). |

---

### 2.2 TypeScript Declaration Statements `[TS]`

| Name | Syntax | Description |
|---|---|---|
| Interface declaration | `interface Point { x: number; y: number; }` | Declares a named structural type. Compile-time only; erased at runtime. Can be merged. |
| Type alias | `type ID = string \| number;` | Declares a named type alias for any type expression. More powerful than interfaces for unions/intersections. |
| Enum declaration | `enum Direction { Up, Down, Left, Right }` | Declares a set of named constants. Numeric or string enums. Compiled to a JS object. |
| Const enum | `const enum Direction { Up = 1 }` | Enum whose members are inlined at use sites; no runtime object is generated. |
| Namespace declaration | `namespace MyNS { export const x = 1; }` | Groups related code under a named object. Compiled to an IIFE. Mostly legacy. |
| Module declaration | `declare module "lodash" { ... }` | Ambient declaration for an external module's types. Used in `.d.ts` files. |
| Ambient declaration | `declare const x: number;` | Declares a variable that exists in the environment but is not defined here (e.g., global from a script tag). |
| Abstract class | `abstract class Shape { abstract area(): number; }` | Class that cannot be instantiated directly; subclasses must implement abstract members. |

---

### 2.3 Control Flow Statements

| Name | Syntax | Description |
|---|---|---|
| `if` statement | `if (cond) { ... }` | Executes a block if the condition is truthy. |
| `if...else` | `if (cond) { ... } else { ... }` | Executes the first block if truthy, the second if falsy. |
| `if...else if...else` | `if (a) {} else if (b) {} else {}` | Chain of conditions; the first truthy branch executes. |
| `switch` statement | `switch (x) { case 1: ...; break; default: ...; }` | Matches an expression against multiple `case` values. Falls through unless `break` is used. |
| Ternary (expression) | `cond ? a : b` | (See expressions.) Also used as a one-liner conditional. |

---

### 2.4 Loop Statements

| Name | Syntax | Description |
|---|---|---|
| `while` loop | `while (cond) { ... }` | Repeats the body as long as `cond` is truthy. Condition checked before each iteration. |
| `do...while` loop | `do { ... } while (cond);` | Repeats the body at least once; condition checked after each iteration. |
| `for` loop | `for (let i = 0; i < n; i++) { ... }` | Classic C-style loop with initializer, condition, and update. |
| `for...in` loop | `for (const key in obj) { ... }` | Iterates over all *enumerable string property keys* of an object, including inherited ones. |
| `for...of` loop | `for (const item of iterable) { ... }` | Iterates over the *values* of any iterable (array, string, Map, Set, generator, etc.). |
| `for await...of` loop | `for await (const item of asyncIterable) { ... }` | Iterates over an async iterable. Must be used inside an `async` function. (ES2018) |

---

### 2.5 Jump Statements

| Name | Syntax | Description |
|---|---|---|
| `break` | `break;`, `break label;` | Exits the nearest enclosing loop or `switch`. With a label, exits the labeled statement. |
| `continue` | `continue;`, `continue label;` | Skips the rest of the current loop iteration and goes to the next. With a label, applies to the labeled loop. |
| `return` | `return;`, `return value;` | Exits the current function, optionally returning a value. |
| `throw` | `throw new Error("msg");` | Throws an exception. Execution jumps to the nearest enclosing `catch` block or unwinds the call stack. |

---

### 2.6 Exception Handling Statements

| Name | Syntax | Description |
|---|---|---|
| `try...catch` | `try { ... } catch (e) { ... }` | Executes the `try` block; if an exception is thrown, `catch` receives it in the binding `e`. |
| `try...finally` | `try { ... } finally { ... }` | The `finally` block always runs after `try`, whether or not an exception was thrown. |
| `try...catch...finally` | `try { ... } catch (e) { ... } finally { ... }` | Combines both: `catch` handles the exception, `finally` runs regardless. |
| Optional catch binding | `try { ... } catch { ... }` | `catch` without a binding variable — when you don't need the error object. (ES2019) |

---

### 2.7 Module Statements

| Name | Syntax | Description |
|---|---|---|
| Named export | `export const x = 1;`, `export function foo() {}` | Exports a binding from the current module by name. |
| Default export | `export default class Foo {}`, `export default 42;` | Exports a single default value from the module. |
| Re-export | `export { foo } from "./mod";` | Re-exports a named binding from another module without importing it locally. |
| Re-export all | `export * from "./mod";` | Re-exports all named exports from another module. |
| Re-export all as namespace | `export * as utils from "./mod";` | Re-exports all named exports under a namespace object. (ES2020) |
| Named import | `import { foo, bar } from "./mod";` | Imports specific named exports. |
| Default import | `import Foo from "./mod";` | Imports the default export. |
| Namespace import | `import * as mod from "./mod";` | Imports all named exports as properties of a namespace object. |
| Side-effect import | `import "./polyfill";` | Runs a module for its side effects only; imports nothing. |
| Type-only import `[TS]` | `import type { Foo } from "./mod";` | Imports only the type; guaranteed erased at runtime. Prevents accidental value usage. |
| Type-only export `[TS]` | `export type { Foo };` | Exports only the type declaration. |

---

### 2.8 Labeled Statement

| Name | Syntax | Description |
|---|---|---|
| Labeled statement | `outer: for (...) { inner: for (...) { break outer; } }` | Attaches a label to a statement (usually a loop). `break` and `continue` can reference the label to control the outer loop. |

---

### 2.9 Miscellaneous Statements

| Name | Syntax | Description |
|---|---|---|
| Expression statement | `foo();`, `x++`, `console.log(x)` | Any expression used as a statement (for its side effects). The value is discarded. |
| Block statement | `{ stmt1; stmt2; }` | Groups multiple statements into one. Creates a new block scope for `let`/`const`. |
| Empty statement | `;` | A no-op. Sometimes used as the body of an empty loop: `while (cond);`. |
| `debugger` statement | `debugger;` | Halts execution and invokes the debugger if one is attached. No-op in production if no debugger is present. |
| `with` statement | `with (obj) { ... }` | Adds `obj` to the scope chain inside the block. **Deprecated**; banned in strict mode. Never use. |
| `"use strict"` directive | `"use strict";` | Enables strict mode for the enclosing script or function. Must be the first statement. |

---

## Part 3 — Class Body Members

Class bodies have their own syntax that is neither purely expressions nor standalone statements.

| Name | Syntax | Description |
|---|---|---|
| Constructor | `constructor(params) { ... }` | Special method called when `new ClassName()` is invoked. |
| Instance method | `greet() { return "hi"; }` | A method on each instance. Dispatched via prototype. |
| Static method | `static create() { return new Foo(); }` | A method on the class itself, not on instances. |
| Getter | `get name() { return this._name; }` | Defines a property that executes a function when read. |
| Setter | `set name(v) { this._name = v; }` | Defines a property that executes a function when written. |
| Static getter/setter | `static get PI() { return 3.14; }` | Getter/setter on the class constructor object. |
| Instance field | `count = 0;` | A field declared directly on each instance. Initialized before the constructor body. (ES2022) |
| Static field | `static count = 0;` | A field on the class constructor itself, shared across all instances. (ES2022) |
| Private field | `#secret = 42;` | A field that is truly private to the class; inaccessible from outside via any means. (ES2022) |
| Private method | `#helper() { ... }` | A method that is truly private to the class. (ES2022) |
| Static private field | `static #count = 0;` | A private field on the class constructor. |
| Static block | `static { this.x = compute(); }` | A block that runs once when the class is evaluated; used for complex static initialization. (ES2022) |
| Abstract method `[TS]` | `abstract speak(): string;` | Declares a method that subclasses must implement. No body allowed. |
| Abstract field `[TS]` | `abstract name: string;` | Declares a field that subclasses must initialize. |
| Access modifiers `[TS]` | `public`, `protected`, `private` | TypeScript compile-time visibility modifiers. `private` is not the same as `#`. |
| `readonly` modifier `[TS]` | `readonly id: number;` | Field can only be assigned in the constructor or at declaration. |
| Parameter property `[TS]` | `constructor(public name: string)` | Shorthand that declares and initializes an instance field from a constructor parameter. |
| Override modifier `[TS]` | `override speak() { ... }` | Marks a method as intentionally overriding a parent method. Compiler error if no parent method exists. |
| Decorator `[TS]` | `@sealed class Foo {}` | Applies a decorator function to a class, method, accessor, or property. (Stage 3 / TS experimental) |

---

## Part 4 — Type System Constructs (TypeScript Only) `[TS]`

These are purely compile-time; they produce no runtime code.

| Name | Syntax | Description |
|---|---|---|
| Primitive types | `number`, `string`, `boolean`, `bigint`, `symbol`, `null`, `undefined`, `void`, `never`, `unknown`, `any`, `object` | Built-in TypeScript types. |
| Literal types | `42`, `"hello"`, `true` | A type that represents exactly one value. |
| Union type | `string \| number` | A value that can be one of several types. |
| Intersection type | `A & B` | A value that satisfies both types simultaneously. |
| Tuple type | `[string, number]` | An array with a fixed number of elements of specific types. |
| Array type | `string[]`, `Array<string>` | An array of a given element type. |
| Function type | `(a: number) => string` | The type of a function with specific parameter and return types. |
| Object type | `{ x: number; y: number }` | An inline structural type for objects. |
| Index signature | `{ [key: string]: number }` | Object with arbitrary string keys all mapping to `number`. |
| Generic type | `Array<T>`, `Promise<T>` | A type parameterized by one or more type variables. |
| Generic constraint | `<T extends object>` | Restricts what types `T` may be. |
| Conditional type | `T extends U ? X : Y` | Selects a type based on whether `T` is assignable to `U`. |
| Mapped type | `{ [K in keyof T]: T[K] }` | Creates a new type by transforming each property of an existing type. |
| Template literal type | `` `${"get" \| "set"}${string}` `` | Constructs string types by combining string literals. |
| `keyof` type operator | `keyof T` | Produces a union of all property name types of `T`. |
| `typeof` type operator | `typeof x` | Produces the TypeScript type of a variable or expression. |
| `infer` keyword | `T extends Promise<infer U> ? U : never` | Declares a type variable to be inferred within a conditional type. |
| Utility types | `Partial<T>`, `Required<T>`, `Readonly<T>`, `Pick<T, K>`, `Omit<T, K>`, `Record<K, V>`, `Exclude<T, U>`, `Extract<T, U>`, `NonNullable<T>`, `ReturnType<F>`, `InstanceType<C>`, `Parameters<F>`, `ConstructorParameters<C>`, `Awaited<T>` | Built-in generic types for common type transformations. |
| Type predicate | `x is string` | Return type of a user-defined type guard function. Narrows the type of the parameter in the calling scope. |
| Assertion function `[TS]` | `asserts x is string` | Return type declaring that if the function returns normally, the assertion holds. |
| `readonly` array | `readonly string[]`, `ReadonlyArray<string>` | An array type whose elements cannot be mutated. |
| `unique symbol` | `declare const sym: unique symbol` | A symbol type that is distinct from all other symbol types. |

---

## Part 5 — Pattern Syntax (Used in Destructuring & Parameters)

| Name | Syntax | Description |
|---|---|---|
| Array pattern | `[a, b]` | Matches array elements positionally. |
| Object pattern | `{ x, y }` | Matches object properties by name. |
| Rest pattern | `...rest` | Collects remaining elements/properties. |
| Default pattern | `x = defaultValue` | Uses `defaultValue` if the matched value is `undefined`. |
| Nested pattern | `{ a: { b } }`, `[[x]]` | Recursively destructures nested structures. |
| Rename pattern | `{ original: alias }` | Binds `original` property to variable named `alias`. |

---

## Appendix — Special Values & Keywords

| Name | Description |
|---|---|
| `NaN` | Not a Number. Result of invalid numeric operations. `typeof NaN === "number"`. |
| `Infinity` | Positive infinity. `-Infinity` is negative infinity. |
| `globalThis` | The global object, regardless of environment (browser, Node.js, worker). (ES2020) |
| `Symbol()` | Creates a unique, immutable primitive symbol value. Used as unique property keys. |
| `Symbol.iterator` | Well-known symbol. Defines the default iterator for an object. |
| `Symbol.asyncIterator` | Well-known symbol. Defines the default async iterator. |
| `Symbol.hasInstance` | Well-known symbol. Customizes `instanceof` behavior. |
| `Symbol.toPrimitive` | Well-known symbol. Customizes type coercion. |
| `Symbol.toStringTag` | Well-known symbol. Customizes `Object.prototype.toString` output. |
