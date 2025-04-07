use crate::emulator::{area_from_address, BitAddressable, Emulator, EmulatorCell, Exception};

use super::Op;
#[derive(Debug, Clone)]
pub struct JmpOp {
    base_r: EmulatorCell,         // Base register index
    target_address: EmulatorCell, // Calculated during evaluate_address
    is_valid_jump: bool,          // Set during evaluate_address
}

impl Op for JmpOp {
    fn decode(ir: EmulatorCell) -> Self {
        // LAYOUT: 1100 | 000 | BaseR | 000000
        let base_r = ir.range(8..6);

        Self {
            base_r,
            target_address: EmulatorCell::new(0), // Initialize to 0
            is_valid_jump: false,                 // Initialize to false
        }
    }

    fn evaluate_address(&mut self, machine_state: &mut Emulator) {
        let base_r_index = self.base_r.get() as usize;
        self.target_address = machine_state.r[base_r_index]; // Get address from register

        // Check memory permissions for the target address
        let target_area = area_from_address(&self.target_address);
        if target_area.can_read(&machine_state.current_privilege_level) {
            self.is_valid_jump = true;
        } else {
            // Privilege violation: Cannot jump to non-readable memory
            machine_state.exception = Some(Exception::new_access_control_violation());
            self.is_valid_jump = false;
            tracing::warn!(
                "JMP/RET Privilege Violation: Attempted jump to non-readable address 0x{:04X} from BaseR R{}",
                self.target_address.get(), base_r_index
            );
        }
    }

    fn execute_operation(&mut self, machine_state: &mut Emulator) {
        // Only update PC if the jump is valid (checked in evaluate_address)
        if self.is_valid_jump {
            machine_state.pc.set(self.target_address.get());
        }
        // If !is_valid_jump, an exception should already be set,
    }
}

use std::fmt;

impl fmt::Display for JmpOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The Display implementation should reflect the state immediately after decode.
        // target_address and is_valid_jump are determined later.
        let base_r_index = self.base_r.get();

        if base_r_index == 7 {
            write!(f, "RET")
        } else {
            write!(f, "JMP R{}", base_r_index)
        }
    }
}
