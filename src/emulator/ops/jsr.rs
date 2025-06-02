use crate::emulator::{area_from_address, BitAddressable, Emulator, EmulatorCell, Exception};

use super::Op;

#[derive(Debug, Clone)]
enum JsrMode {
    Relative { pc_offset: EmulatorCell },
    Register { base_r: EmulatorCell },
}

#[derive(Debug, Clone)]
pub struct JsrOp {
    mode: JsrMode,
    target_address: EmulatorCell, // Calculated during evaluate_address
    is_valid_jump: bool,          // Set during evaluate_address
}

impl Op for JsrOp {
    fn decode(ir: EmulatorCell) -> Self {
        // LAYOUT: 0100 | ToggleBit | rest of bits (PCoffset11 or BaseR)

        let mode = if ir.index(11).get() == 1 {
            // JSR: Use PC-relative addressing
            // Extract and sign-extend PCoffset11
            let pc_offset = ir.range(10..0).sext(10);
            JsrMode::Relative { pc_offset }
        } else {
            // JSRR: Get address from base register
            let base_r = ir.range(8..6);
            JsrMode::Register { base_r }
        };

        Self {
            mode,
            target_address: EmulatorCell::new(0),
            is_valid_jump: false,
        }
    }

    fn evaluate_address(&mut self, machine_state: &mut Emulator) {
        // PC has already been incremented in fetch phase
        let return_pc = machine_state.pc; // This is the address of the *next* instruction

        // Save return address in R7
        machine_state.r[7] = return_pc;

        // Calculate target address based on mode
        match &self.mode {
            JsrMode::Relative { pc_offset } => {
                // Target is PC + offset
                self.target_address =
                    EmulatorCell::new(return_pc.get().wrapping_add(pc_offset.get()));
            }
            JsrMode::Register { base_r } => {
                // Target is the value in the base register
                let base_r_index = base_r.get() as usize;
                self.target_address = machine_state.r[base_r_index];
            }
        }

        // Check memory permissions for the target address
        let target_area = area_from_address(&self.target_address);
        if target_area.can_read(&machine_state.priv_level()) {
            self.is_valid_jump = true;
        } else {
            // Privilege violation: Cannot jump to non-readable memory
            machine_state.exception = Some(Exception::new_access_control_violation());
            self.is_valid_jump = false;
            tracing::warn!(
                "JSR/JSRR Privilege Violation: Attempted jump to non-readable address 0x{:04X}",
                self.target_address.get()
            );
        }
    }

    fn execute_operation(&mut self, machine_state: &mut Emulator) {
        // Only update PC if the jump is valid (checked in evaluate_address)
        if self.is_valid_jump {
            machine_state.pc.set(self.target_address.get());
        }
        // If !is_valid_jump, an exception should already be set.
    }
}

use std::fmt;

impl fmt::Display for JsrOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display implementation based on the state after decode.
        match &self.mode {
            JsrMode::Relative { pc_offset } => {
                // JSR: Display with PC-relative offset
                let offset_val = pc_offset.get() as i16; // Cast to signed for decimal display
                write!(
                    f,
                    "JSR #{} (x{:03X})",
                    offset_val,
                    pc_offset.get() & 0x7FF // Mask to 11 bits for hex
                )
            }
            JsrMode::Register { base_r } => {
                // JSRR: Display with base register
                write!(f, "JSRR R{}", base_r.get())
            }
        }
    }
}
