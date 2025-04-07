use crate::emulator::{area_from_address, BitAddressable, Emulator, EmulatorCell, Exception};

use super::Op;

#[derive(Debug, Clone)]
pub struct LdOp {
    dr: EmulatorCell,                // Destination Register index
    pc_offset: EmulatorCell,         // PCoffset9 (sign-extended)
    effective_address: EmulatorCell, // Calculated address
    is_valid_load: bool,             // Flag if the address is valid to read from
}

impl Op for LdOp {
    fn decode(ir: EmulatorCell) -> Self {
        // LAYOUT: 0010 | DR | PCoffset9
        let dr = ir.range(11..9);
        // Extract and sign-extend PCoffset9 during decode
        let pc_offset = ir.range(8..0).sext(8);

        Self {
            dr,
            pc_offset,
            effective_address: EmulatorCell::new(0),
            is_valid_load: false,
        }
    }

    fn evaluate_address(&mut self, machine_state: &mut Emulator) {
        // Calculate effective address: PC + SEXT(PCoffset9)
        // PC was already incremented during the fetch phase
        let current_pc = machine_state.pc;
        let effective_addr_val = current_pc.get().wrapping_add(self.pc_offset.get());
        self.effective_address.set(effective_addr_val);

        // Check memory read permissions
        let target_area = area_from_address(&self.effective_address);
        if target_area.can_read(&machine_state.current_privilege_level) {
            // Mark the load as valid, MAR will be set in fetch_operands
            self.is_valid_load = true;
        } else {
            // Privilege violation: Cannot read from this memory location
            machine_state.exception = Some(Exception::new_access_control_violation());
            self.is_valid_load = false;
            // Optional: warn logging if needed
            tracing::warn!(
                address = format!("0x{:04X}", self.effective_address.get()),
                "LD Privilege Violation: Cannot read from address"
            );
        }
    }

    fn fetch_operands(&mut self, machine_state: &mut Emulator) -> bool {
        // If the address is valid (checked in evaluate_address), set MAR
        // for the memory read that will happen implicitly *after* this phase.
        if self.is_valid_load {
            machine_state.mar = self.effective_address;
        }
        // The actual memory read (MDR <- Mem[MAR]) happens after this phase if MAR is set.

        false
    }

    fn execute_operation(&mut self, _machine_state: &mut Emulator) {
        // No specific execution/ALU operation for LD.
        // The main work (memory read and register write) happens around fetch/store phases.
    }

    fn store_result(&mut self, machine_state: &mut Emulator) {
        // Only perform the register write if the load address was valid
        if self.is_valid_load {
            // MDR contains the value read from memory (implicitly loaded after fetch_operands)
            let value_loaded = machine_state.mdr;
            let dr_index = self.dr.get() as usize;

            // Write the loaded value into the destination register
            machine_state.r[dr_index] = value_loaded;

            // Update condition codes based on the value written to the register
            machine_state.update_flags(dr_index);
        }
        // If !is_valid_load, an exception was set in evaluate_address, and the store is skipped.
    }
}

use std::fmt;

impl fmt::Display for LdOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display the state immediately after decode.
        // effective_address and is_valid_load are calculated later.
        let dr_index = self.dr.get();
        let offset_val = self.pc_offset.get() as i16; // Cast to signed for display
        let offset_hex = self.pc_offset.get() & 0x1FF; // Mask to 9 bits for hex

        write!(f, "LD R{}, #{} (x{:03X})", dr_index, offset_val, offset_hex)
    }
}
