use crate::types::{MirInstr, MirOperand, MirReg};

pub static ACQUISITION_VERBS: &[&str] = &[
    "open", "connect", "create", "acquire", "lock", "begin",
    "start", "spawn", "fork", "listen", "bind", "attach",
    "getConnection", "getClient", "getChannel", "checkout", "lease", "borrow",
];

pub static RELEASE_VERBS: &[&str] = &[
    "close", "destroy", "dispose", "release", "unlock", "end",
    "stop", "terminate", "kill", "disconnect", "detach", "free",
    "commit", "rollback", "abort", "finish", "done", "cleanup",
    "return", "checkin",
];

pub fn is_acquisition_call(instr: &MirInstr) -> Option<(MirReg, String)> {
    match instr {
        MirInstr::CallDirect(dest, name, _args) | MirInstr::CallPure(dest, name, _args) => {
            let bare_name = if name.starts_with("__bs_") { &name[5..] } else { name.as_str() };
            if ACQUISITION_VERBS.contains(&bare_name) {
                return Some((*dest, bare_name.to_string()));
            }
            // For methods, usually CallVTable or CallClosure, we might need more checks
            None
        }
        // Add support for method calls once we have enough context
        _ => None,
    }
}

pub fn is_release_call(instr: &MirInstr) -> Option<(MirReg, String)> {
    match instr {
        MirInstr::CallDirect(_dest, name, args) | MirInstr::CallPure(_dest, name, args) => {
            let bare_name = if name.starts_with("__bs_") { &name[5..] } else { name.as_str() };
            if RELEASE_VERBS.contains(&bare_name) {
                // Return the first argument as the target (the object being released)
                if let Some(MirOperand::Reg(r)) = args.first() {
                    return Some((*r, name.clone())); // Keep the full name for the release function
                }
            }
            None
        }
        // In BinScript, VTable/Closure calls are used for methods
        MirInstr::CallClosure(_dest, _callee, _args) | MirInstr::CallVTable(_dest, _callee, _, _args) => {
             // For closures/vtables we might not have the name easily accessible in the instruction itself
             // We'd need to look at property loads. For now, rely on CallDirect if that's how we compile `close(res)`
             None
        }
        _ => None,
    }
}
