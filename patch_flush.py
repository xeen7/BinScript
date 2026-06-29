import re
content = open("rt-stubs/src/raii/scope_guard.rs").read()
content = content.replace("while stack.len() > frame_base as usize {", "libc::printf(b\"Flushing to %d\\n\\0\".as_ptr() as *const _, frame_base);\n        while stack.len() > frame_base as usize {\n            libc::printf(b\"Loop len=%d\\n\\0\".as_ptr() as *const _, stack.len());")
open("rt-stubs/src/raii/scope_guard.rs", "w").write(content)
