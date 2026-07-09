with open("crates/codegen-llvm/src/codegen/instr/calls/call_direct.rs", "r") as f:
    code = f.read()

intercept = """        match instr {
            MirInstr::CallDirect(d, name, args) | MirInstr::CallPure(d, name, args) => {
                let mut resolved_name = name.clone();
                let dest_class = self.funcs_mem_classes.get(&self.current_fn_name.clone().unwrap())
                    .and_then(|classes| classes.get(d));
                if dest_class == Some(&mir::MemClass::Owned) {
                    if name == "__bs_string_concat" {
                        resolved_name = "__bs_string_concat_owned".to_string();
                    } else if name == "__bs_number_to_string" {
                        resolved_name = "__bs_number_to_string_owned".to_string();
                    } else if name == "__bs_boolean_to_string" {
                        resolved_name = "__bs_boolean_to_string_owned".to_string();
                    }
                }
                
                let fn_val = self.funcs.get(&resolved_name).copied().ok_or_else(|| {"""

code = code.replace("""        match instr {
            MirInstr::CallDirect(d, name, args) | MirInstr::CallPure(d, name, args) => {
                let fn_val = self.funcs.get(name).copied().ok_or_else(|| {""", intercept)

with open("crates/codegen-llvm/src/codegen/instr/calls/call_direct.rs", "w") as f:
    f.write(code)
