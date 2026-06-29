import re
content = open("rt-stubs/src/exception/mod.rs").read()
content = content.replace("libc::printf(b\"Calling _Unwind_RaiseException\\n\\0\".as_ptr() as *const _);", "libc::printf(b\"Calling _Unwind_RaiseException\\n\\0\".as_ptr() as *const _);\n    libc::fflush(std::ptr::null_mut());")
content = content.replace("libc::printf(b\"_Unwind_RaiseException FAILED!\\n\\0\".as_ptr() as *const _);", "libc::printf(b\"_Unwind_RaiseException FAILED!\\n\\0\".as_ptr() as *const _);\n    libc::fflush(std::ptr::null_mut());")
open("rt-stubs/src/exception/mod.rs", "w").write(content)

content = open("rt-stubs/src/raii/scope_guard.rs").read()
content = content.replace("libc::printf(b\"Flushing to %d\\n\\0\".as_ptr() as *const _, frame_base);", "libc::printf(b\"Flushing to %d\\n\\0\".as_ptr() as *const _, frame_base);\n        libc::fflush(std::ptr::null_mut());")
content = content.replace("libc::printf(b\"Loop len=%d\\n\\0\".as_ptr() as *const _, stack.len());", "libc::printf(b\"Loop len=%d\\n\\0\".as_ptr() as *const _, stack.len());\n            libc::fflush(std::ptr::null_mut());")
open("rt-stubs/src/raii/scope_guard.rs", "w").write(content)

