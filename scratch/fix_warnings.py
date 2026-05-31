import os

# 1. Unreachable pattern in crates/hir/src/lower/operators.rs:45
with open('crates/hir/src/lower/operators.rs', 'r') as f:
    ops = f.read()
# Let's just remove the `_ => crate::types::UnaryOp::Neg,` if it's the last arm and it's unreachable
ops = ops.replace('_ => crate::types::UnaryOp::Neg,\n', '')
with open('crates/hir/src/lower/operators.rs', 'w') as f:
    f.write(ops)

# 2. Unreachable pattern in crates/hir/src/lower/expr/call.rs:257
with open('crates/hir/src/lower/expr/call.rs', 'r') as f:
    call = f.read()
call = call.replace('                        _ => Err(CompileError::Lowering {\n                            msg: "Unsupported member property type in call".into(),\n                            span: call.span,\n                        }),\n', '')
with open('crates/hir/src/lower/expr/call.rs', 'w') as f:
    f.write(call)

# 3. Variable mutability in crates/mir/src/lower/builtins/json.rs:10
with open('crates/mir/src/lower/builtins/json.rs', 'r') as f:
    json_mir = f.read()
json_mir = json_mir.replace('mut mir_args: Vec<MirOperand>,', 'mir_args: Vec<MirOperand>,')
with open('crates/mir/src/lower/builtins/json.rs', 'w') as f:
    f.write(json_mir)

# 4. array_forEach_wrapper in rt-stubs/src/dynamic_call/dispatchers.rs
with open('rt-stubs/src/dynamic_call/dispatchers.rs', 'r') as f:
    disp = f.read()
disp = disp.replace('array_forEach_wrapper', 'array_for_each_wrapper')
with open('rt-stubs/src/dynamic_call/dispatchers.rs', 'w') as f:
    f.write(disp)
    
# We also need to fix references to array_forEach_wrapper in rt-stubs/src/dynamic_call/mod.rs
with open('rt-stubs/src/dynamic_call/mod.rs', 'r') as f:
    mod = f.read()
mod = mod.replace('array_forEach_wrapper', 'array_for_each_wrapper')
with open('rt-stubs/src/dynamic_call/mod.rs', 'w') as f:
    f.write(mod)
    
# And wherever it's registered
def replace_in_files(replacements, ext='.rs'):
    for root, _, files in os.walk('rt-stubs/src'):
        for file in files:
            if not file.endswith(ext): continue
            path = os.path.join(root, file)
            with open(path, 'r') as f:
                content = f.read()
            original = content
            for old, new in replacements:
                content = content.replace(old, new)
            if content != original:
                with open(path, 'w') as f:
                    f.write(content)

replace_in_files([('array_forEach_wrapper', 'array_for_each_wrapper')])

# 5. Math unused constants
with open('rt-stubs/src/objects/builtins/math.rs', 'r') as f:
    math = f.read()
math = math.replace('const TAG_MASK: u64 = 0xFFFF_0000_0000_0000;\n', '')
math = math.replace('const TAG_STRING: u64 = 0xFFF7_0000_0000_0000;\n', '')
with open('rt-stubs/src/objects/builtins/math.rs', 'w') as f:
    f.write(math)

print("Fixed warnings manually!")
