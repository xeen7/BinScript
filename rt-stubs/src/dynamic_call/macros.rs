#[macro_export]
macro_rules! dispatch_0_args {
    ($name:ident, $method_name:expr, $arr_fn:expr, $str_fn:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(mut recv: u64, idx_boxed: u64) -> u64 {
            let idx = f64::from_bits(idx_boxed) as i32;
            let mut tag = recv & TAG_MASK;
            if tag == TAG_OBJECT {
                let payload = recv & PAYLOAD_MASK;
                if payload != 0 {
                    let obj_ptr = payload as *mut u8;
                    let vtable_ptr = *(obj_ptr as *const *const $crate::VTable);
                    if !vtable_ptr.is_null() {
                        let name_cstr = std::ffi::CStr::from_ptr((*vtable_ptr).name as *const libc::c_char);
                        let name_bytes = name_cstr.to_bytes();
                        if name_bytes == b"String" {
                            if let Some(prim) = $crate::objects::dynamic_props::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        } else if name_bytes == b"Number" {
                            if let Some(prim) = $crate::objects::dynamic_props::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        } else if name_bytes == b"Boolean" {
                            if let Some(prim) = $crate::objects::dynamic_props::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        } else if name_bytes == b"Date" {
                            if $method_name == "getTime" || $method_name == "valueOf" {
                                return $crate::objects::date::__bs_date_getTime(recv);
                            } else if $method_name == "getFullYear" {
                                return $crate::objects::date::__bs_date_getFullYear(recv);
                            } else if $method_name == "getMonth" {
                                return $crate::objects::date::__bs_date_getMonth(recv);
                            } else if $method_name == "getDate" {
                                return $crate::objects::date::__bs_date_getDate(recv);
                            } else if $method_name == "getHours" {
                                return $crate::objects::date::__bs_date_getHours(recv);
                            } else if $method_name == "getMinutes" {
                                return $crate::objects::date::__bs_date_getMinutes(recv);
                            } else if $method_name == "getSeconds" {
                                return $crate::objects::date::__bs_date_getSeconds(recv);
                            } else if $method_name == "toString" {
                                if let Some(prim) = $crate::objects::dynamic_props::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                    let ms = f64::from_bits(prim);
                                    return $crate::types::string_utils::create_tagged_string(&$crate::objects::date::date_to_string(ms));
                                }
                            }
                        }
                    }
                }
            }
            if $method_name == "toString" {
                if tag == TAG_STRING {
                    return recv;
                } else if tag == TAG_ARRAY {
                    return $crate::types::coercion::__bs_String(recv);
                }
            } else if $method_name == "valueOf" {
                if tag == TAG_STRING || tag == TAG_ARRAY {
                    return recv;
                }
            }

            if tag == TAG_ARRAY {
                let f: unsafe extern "C" fn(u64) -> u64 = $arr_fn;
                f(recv)
            } else if tag == TAG_STRING {
                let f: unsafe extern "C" fn(u64) -> u64 = $str_fn;
                f(recv)
            } else if tag == TAG_OBJECT {
                if let Some(method_ptr) = $crate::dynamic_call::helpers::get_user_method(recv, idx) {
                    let f: unsafe extern "C" fn(u64) -> u64 = std::mem::transmute(method_ptr);
                    f(recv)
                } else {
                    if $method_name == "toString" {
                        return $crate::types::string_utils::create_tagged_string("[object Object]");
                    }
                    if $method_name == "valueOf" {
                        return recv;
                    }
                    panic!("Method not found on user object");
                }
            } else {
                if $method_name == "toString" {
                    return $crate::types::coercion::__bs_String(recv);
                } else if $method_name == "valueOf" {
                    return recv;
                }
                panic!("Method called on incompatible receiver type");
            }
        }
    };
}

#[macro_export]
macro_rules! dispatch_1_arg {
    ($name:ident, $method_name:expr, $arr_fn:expr, $str_fn:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(mut recv: u64, arg1: u64, idx_boxed: u64) -> u64 {
            let idx = f64::from_bits(idx_boxed) as i32;
            let mut tag = recv & TAG_MASK;
            if tag == TAG_OBJECT {
                let payload = recv & PAYLOAD_MASK;
                if payload != 0 {
                    let obj_ptr = payload as *mut u8;
                    let vtable_ptr = *(obj_ptr as *const *const $crate::VTable);
                    if !vtable_ptr.is_null() {
                        let name_cstr = std::ffi::CStr::from_ptr((*vtable_ptr).name as *const libc::c_char);
                        let name_bytes = name_cstr.to_bytes();
                        if name_bytes == b"String" {
                            if let Some(prim) = $crate::objects::dynamic_props::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        } else if name_bytes == b"Number" {
                            if let Some(prim) = $crate::objects::dynamic_props::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        } else if name_bytes == b"Boolean" {
                            if let Some(prim) = $crate::objects::dynamic_props::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        }
                    }
                }
            }
            if tag == TAG_ARRAY {
                let f: unsafe extern "C" fn(u64, u64) -> u64 = $arr_fn;
                f(recv, arg1)
            } else if tag == TAG_STRING {
                let f: unsafe extern "C" fn(u64, u64) -> u64 = $str_fn;
                f(recv, arg1)
            } else if tag == TAG_OBJECT {
                if let Some(method_ptr) = $crate::dynamic_call::helpers::get_user_method(recv, idx) {
                    let f: unsafe extern "C" fn(u64, u64) -> u64 = std::mem::transmute(method_ptr);
                    f(recv, arg1)
                } else {
                    panic!("Method not found on user object");
                }
            } else {
                panic!("Method called on incompatible receiver type");
            }
        }
    };
}

#[macro_export]
macro_rules! dispatch_2_args {
    ($name:ident, $method_name:expr, $arr_fn:expr, $str_fn:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(mut recv: u64, arg1: u64, arg2: u64, idx_boxed: u64) -> u64 {
            let idx = f64::from_bits(idx_boxed) as i32;
            let mut tag = recv & TAG_MASK;
            if tag == TAG_OBJECT {
                let payload = recv & PAYLOAD_MASK;
                if payload != 0 {
                    let obj_ptr = payload as *mut u8;
                    let vtable_ptr = *(obj_ptr as *const *const $crate::VTable);
                    if !vtable_ptr.is_null() {
                        let name_cstr = std::ffi::CStr::from_ptr((*vtable_ptr).name as *const libc::c_char);
                        let name_bytes = name_cstr.to_bytes();
                        if name_bytes == b"String" {
                            if let Some(prim) = $crate::objects::dynamic_props::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        } else if name_bytes == b"Number" {
                            if let Some(prim) = $crate::objects::dynamic_props::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        } else if name_bytes == b"Boolean" {
                            if let Some(prim) = $crate::objects::dynamic_props::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        }
                    }
                }
            }
            if tag == TAG_ARRAY {
                let f: unsafe extern "C" fn(u64, u64, u64) -> u64 = $arr_fn;
                f(recv, arg1, arg2)
            } else if tag == TAG_STRING {
                let f: unsafe extern "C" fn(u64, u64, u64) -> u64 = $str_fn;
                f(recv, arg1, arg2)
            } else if tag == TAG_OBJECT {
                if let Some(method_ptr) = $crate::dynamic_call::helpers::get_user_method(recv, idx) {
                    let f: unsafe extern "C" fn(u64, u64, u64) -> u64 = std::mem::transmute(method_ptr);
                    f(recv, arg1, arg2)
                } else {
                    panic!("Method not found on user object");
                }
            } else {
                panic!("Method called on incompatible receiver type");
            }
        }
    };
}

#[macro_export]
macro_rules! dispatch_3_args {
    ($name:ident, $method_name:expr, $arr_fn:expr, $str_fn:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(mut recv: u64, arg1: u64, arg2: u64, arg3: u64, idx_boxed: u64) -> u64 {
            let idx = f64::from_bits(idx_boxed) as i32;
            let mut tag = recv & TAG_MASK;
            if tag == TAG_OBJECT {
                let payload = recv & PAYLOAD_MASK;
                if payload != 0 {
                    let obj_ptr = payload as *mut u8;
                    let vtable_ptr = *(obj_ptr as *const *const $crate::VTable);
                    if !vtable_ptr.is_null() {
                        let name_cstr = std::ffi::CStr::from_ptr((*vtable_ptr).name as *const libc::c_char);
                        let name_bytes = name_cstr.to_bytes();
                        if name_bytes == b"String" {
                            if let Some(prim) = $crate::objects::dynamic_props::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        } else if name_bytes == b"Number" {
                            if let Some(prim) = $crate::objects::dynamic_props::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        } else if name_bytes == b"Boolean" {
                            if let Some(prim) = $crate::objects::dynamic_props::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
                                recv = prim;
                                tag = recv & TAG_MASK;
                            }
                        }
                    }
                }
            }
            if tag == TAG_ARRAY {
                let f: unsafe extern "C" fn(u64, u64, u64, u64) -> u64 = $arr_fn;
                f(recv, arg1, arg2, arg3)
            } else if tag == TAG_STRING {
                let f: unsafe extern "C" fn(u64, u64, u64, u64) -> u64 = $str_fn;
                f(recv, arg1, arg2, arg3)
            } else if tag == TAG_OBJECT {
                if let Some(method_ptr) = $crate::dynamic_call::helpers::get_user_method(recv, idx) {
                    let f: unsafe extern "C" fn(u64, u64, u64, u64) -> u64 = std::mem::transmute(method_ptr);
                    f(recv, arg1, arg2, arg3)
                } else {
                    panic!("Method not found on user object");
                }
            } else {
                panic!("Method called on incompatible receiver type");
            }
        }
    };
}

