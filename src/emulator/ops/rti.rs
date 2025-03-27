use crate::emulator::{Emulator, Exception, PrivilegeLevel};

use super::Op;
#[derive(Debug)]
pub struct RtiOp;

impl Op for RtiOp {
    fn prepare_memory_access(&self, _machine_state: &mut Emulator) {
        // RTI doesn't need extra memory access preparation in this implementation
    }

    fn execute(&self, machine_state: &mut Emulator) {
        if matches!(machine_state.current_privilege_level, PrivilegeLevel::User) {
            machine_state.exception = Some(Exception::new_privilege_violation())
        } else {
            unreachable!("we do not have an OS yet (TODO)")
        }
    }
}
