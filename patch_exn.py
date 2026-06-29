import re
content = open("rt-stubs/src/exception/mod.rs").read()
content = content.replace("pub unsafe extern \"C\" fn __bs_get_exception_value(exn_ptr: *mut _Unwind_Exception) -> u64 {", "pub unsafe extern \"C\" fn __bs_get_exception_value(exn_ptr: *mut _Unwind_Exception) -> u64 {\n    libc::printf(b\"In get_exception_value\\n\\0\".as_ptr() as *const _);\n    libc::fflush(std::ptr::null_mut());")
open("rt-stubs/src/exception/mod.rs", "w").write(content)
