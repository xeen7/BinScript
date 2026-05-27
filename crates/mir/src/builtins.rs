//! Built-in function identifiers for the MIR layer.

/// Built-in functions that map to compiler-emitted LLVM stubs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltinFn {
    /// `console.log(…)` — prints values to stdout.
    ConsoleLog,
    /// `__bs_generator_next` — resumes a generator.
    GeneratorNext,
    /// `__bs_generator_is_done` — checks if generator is exhausted.
    GeneratorIsDone,
    /// `__bs_promise_all_2` — waits for 2 promises.
    PromiseAll2,
    /// `__bs_promise_race_2` — races 2 promises.
    PromiseRace2,
    /// `__bs_json_parse_lazy` — parses JSON lazily.
    JsonParseLazy,
    // --- Array built-ins ---
    ArrayNew,
    ArrayFrom,
    ArrayPush,
    ArrayPop,
    ArrayGet,
    ArraySet,
    ArrayLength,
    ArraySlice,
    ArrayIndexOf,
    ArrayIncludes,
    ArrayJoin,
    ArrayReverse,
    ArrayConcat,
    ArrayFill,
    ArrayIsArray,
    ArrayForEach,
    ArrayMap,
    ArrayFilter,
    ArrayFind,
    ArrayFindIndex,
    ArrayEvery,
    ArraySome,
    ArrayReduce,
}
