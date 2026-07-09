#[derive(Debug, Clone, Default)]
pub struct NativeSignature {
    /// True if the function is guaranteed to not capture any arguments globally.
    pub is_safe_stub: bool,
    /// True if the function returns a newly allocated, unaliased object.
    pub returns_fresh_allocation: bool,
    /// True if the function always returns a primitive (Number, Boolean, String primitive, Undefined).
    pub returns_primitive: bool,
    /// Dependency: Some((src_arg_idx, dest_arg_idx)). 
    /// E.g., for `push(arr, val)`, (1, 0) means val flows into arr.
    pub argument_flow: Option<(usize, usize)>,
}

impl NativeSignature {
    pub fn get(target: &str) -> Option<Self> {
        let mut sig = NativeSignature::default();
        
        // String methods (non-dynamic)
        if target.starts_with("__bs_string_") {
            sig.is_safe_stub = true;
            sig.returns_fresh_allocation = true; // String returns are fresh
            return Some(sig);
        }
        
        // Dynamic method calls (__bs_call_...)
        if let Some(method_name) = target.strip_prefix("__bs_call_") {
            sig.is_safe_stub = true;
            
            match method_name {
                "push" | "unshift" => {
                    println!("NativeSignature matched push: {}", method_name);
                    sig.argument_flow = Some((1, 0)); // arg 1 flows into receiver (arg 0)
                    sig.returns_primitive = true; // returns length
                }
                "map" | "filter" | "slice" | "concat" | "substring" | "split" | "replace" | "trim" | "toLowerCase" | "toUpperCase" => {
                    sig.returns_fresh_allocation = true;
                }
                "indexOf" | "includes" | "join" | "every" | "some" | "length" | "charCodeAt" | "charAt" => {
                    sig.returns_primitive = true;
                }
                "then" | "catch" | "finally" => {
                    sig.is_safe_stub = false; // Callbacks escape globally
                    sig.returns_fresh_allocation = true; // Returns a new Promise
                }
                _ => {}
            }
            return Some(sig);
        }
        
        // Object methods
        if target.starts_with("__bs_object_") {
            sig.is_safe_stub = true;
            if target == "__bs_object_create" || target == "__bs_object_keys" || target == "__bs_object_values" || target == "__bs_object_entries" {
                sig.returns_fresh_allocation = true;
            }
            return Some(sig);
        }

        // Array methods & Index operations
        if target.starts_with("__bs_array_") || target.starts_with("__bs_index_") {
            sig.is_safe_stub = true;
            
            match target {
                "__bs_array_new" => {
                    sig.returns_fresh_allocation = true;
                }
                "__bs_array_slice" | "__bs_array_concat" | "__bs_array_map" | "__bs_array_filter" => {
                    sig.returns_fresh_allocation = true;
                }
                "__bs_array_push" | "__bs_array_unshift" => {
                    sig.argument_flow = Some((1, 0)); // arg 1 flows into arg 0
                    sig.returns_primitive = true; // returns new length
                }
                "__bs_array_set" | "__bs_index_set" => {
                    sig.argument_flow = Some((2, 0)); // arg 2 flows into arg 0
                    sig.returns_primitive = true; // returns boolean/undefined
                }
                _ => {}
            }
            return Some(sig);
        }
        
        // Math and Number functions
        if target.starts_with("__bs_math_") || 
           target.starts_with("__bs_number_") ||
           target == "__bs_isFinite" || 
           target == "__bs_isNaN" ||
           target == "__bs_parseInt" ||
           target == "__bs_parseFloat" {
            sig.is_safe_stub = true;
            sig.returns_primitive = true;
            return Some(sig);
        }

        // JSON methods
        if target.starts_with("__bs_json_") {
            sig.is_safe_stub = true;
            if target == "__bs_json_parse" {
                sig.returns_fresh_allocation = true;
            }
            return Some(sig);
        }
        
        // Promise methods
        if target.starts_with("__bs_promise_") {
            sig.is_safe_stub = true; // Resolve/Reject are safe, but `then` captures callbacks globally. Currently handled elsewhere if custom.
            return Some(sig);
        }

        // Date methods
        if target.starts_with("__bs_date_") {
            sig.is_safe_stub = true;
            return Some(sig);
        }
        
        // Console methods
        if target.starts_with("__bs_console_") {
            sig.is_safe_stub = true;
            return Some(sig);
        }

        
        // Globals & Core Runtime
        if target == "__bs_get_globalThis" || target == "__bs_get_Symbol_global" || target == "__bs_cleanup_global_this" ||
           target == "__bs_get_property" || target == "__bs_set_property" || target == "__bs_prop_get" || target == "__bs_prop_set" ||
           target == "__bs_to_string" || target == "__bs_to_number" || target == "__bs_to_boolean" ||
           target == "__bs_strict_eq" || target == "__bs_loose_eq" || target == "__bs_strict_ne" || target == "__bs_loose_ne" ||
           target == "__bs_type_of" || target == "__bs_instance_of" || target == "__bs_typeof" || target == "__bs_instanceof" ||
           target == "__bs_add" || target == "__bs_sub" || target == "__bs_is_nullish" || target == "__bs_exp" || 
           target == "__bs_in" || target == "__bs_delete_prop" ||
           target == "__bs_encodeURI" || target == "__bs_decodeURI" || target == "__bs_encodeURIComponent" || target == "__bs_decodeURIComponent" ||
           target == "__bs_Symbol" || target == "__bs_Symbol_0" || target == "__bs_Symbol_1" ||
           target == "__bs_String" || target == "__bs_Number" || target == "__bs_Boolean" || target == "__bs_Object" || target == "__bs_Date" {
            sig.is_safe_stub = true;
            return Some(sig);
        }

        // Additional constructors and types
        if target.starts_with("__bs_String_") || target.starts_with("__bs_Number_") || target.starts_with("__bs_Boolean_") ||
           target.starts_with("__bs_Error_") || target.starts_with("__bs_TypeError_") || target.starts_with("__bs_RangeError_") ||
           target.starts_with("__bs_ReferenceError_") || target.starts_with("__bs_SyntaxError_") || target.starts_with("__bs_URIError_") ||
           target.starts_with("__bs_Set_") || target.starts_with("__bs_Map_") || target.starts_with("__bs_WeakMap_") || target.starts_with("__bs_WeakSet_") ||
           target.starts_with("__bs_RegExp_")  {
            sig.is_safe_stub = true;
            sig.returns_fresh_allocation = true;
            return Some(sig);
        }
        
        // Object constructors — __bs_Object_new_0 allocates fresh, but
        // __bs_Object_new_1 may return its input unchanged (e.g. new Object(existingObj) === existingObj).
        if target.starts_with("__bs_Object_") {
            sig.is_safe_stub = true;
            if target == "__bs_Object_new_0" || target == "__bs_Object_new" {
                sig.returns_fresh_allocation = true;
            }
            // __bs_Object_new_1 is intentionally NOT returns_fresh_allocation
            return Some(sig);
        }
        
        // System and Generators
        if target.starts_with("__bs_fs_") || target.starts_with("__bs_path_") || target.starts_with("__bs_os_") || target.starts_with("__bs_generator_") {
            sig.is_safe_stub = true;
            if target == "__bs_fs_exists_sync" || target == "__bs_generator_is_done" {
                sig.returns_primitive = true;
            } else if target != "__bs_fs_write_file_sync" {
                sig.returns_fresh_allocation = true;
            }
            return Some(sig);
        }

        // Constructors
        if target.ends_with("_constructor") {
            sig.is_safe_stub = true;
            sig.returns_fresh_allocation = true;
            return Some(sig);
        }
        
        None
    }
}
