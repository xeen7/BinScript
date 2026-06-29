import re
content = open("rt-stubs/src/exception/mod.rs").read()
content = content.replace("let res = _Unwind_RaiseException", "libc::printf(b\"Calling _Unwind_RaiseException\\n\\0\".as_ptr() as *const _);\n    let res = _Unwind_RaiseException")
content = content.replace("eprintln!(\"_Unwind_RaiseException failed", "libc::printf(b\"_Unwind_RaiseException FAILED!\\n\\0\".as_ptr() as *const _);\n    eprintln!(\"_Unwind_RaiseException failed")
open("rt-stubs/src/exception/mod.rs", "w").write(content)
