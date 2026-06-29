import re

content = open("rt-stubs/src/exception/mod.rs").read()
# Replace __bs_throw with a trace
if "libc::printf(b\"__bs_throw called\\n\\0\".as_ptr() as *const _);" not in content:
    content = content.replace("pub unsafe extern \"C\" fn __bs_throw(value: u64) -> ! {", "pub unsafe extern \"C\" fn __bs_throw(value: u64) -> ! {\n    libc::printf(b\"__bs_throw called\\n\\0\".as_ptr() as *const _);\n    libc::fflush(std::ptr::null_mut());")
open("rt-stubs/src/exception/mod.rs", "w").write(content)

content = open("rt-stubs/src/objects/builtins/error.rs").read()
if "libc::printf(b\"__bs_Error_new called\\n\\0\".as_ptr() as *const _);" not in content:
    content = content.replace("pub unsafe extern \"C\" fn __bs_Error_new(message_tagged: u64) -> u64 {", "pub unsafe extern \"C\" fn __bs_Error_new(message_tagged: u64) -> u64 {\n    libc::printf(b\"__bs_Error_new called\\n\\0\".as_ptr() as *const _);\n    libc::fflush(std::ptr::null_mut());")
open("rt-stubs/src/objects/builtins/error.rs", "w").write(content)

content = open("rt-stubs/src/raii/scope_guard.rs").read()
# Restore the while loop
content = content.replace("libc::printf(b\"Flushing to %d\\n\\0\".as_ptr() as *const _, frame_base);\n        libc::fflush(std::ptr::null_mut());", "libc::printf(b\"Flushing to %d\\n\\0\".as_ptr() as *const _, frame_base);\n        libc::fflush(std::ptr::null_mut());\n        while stack.len() > frame_base as usize {")
open("rt-stubs/src/raii/scope_guard.rs", "w").write(content)

