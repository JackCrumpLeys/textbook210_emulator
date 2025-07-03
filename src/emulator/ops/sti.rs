use crate::emulator::{area_from_address, BitAddressable, Emulator, EmulatorCell, Exception};

use super::Op;

#[derive(Debug, Clone)]
/// Store some register value at `mem[mem[pc+pc_offset]]`
pub struct StiOp {
    pub sr: EmulatorCell,               // Source Register index
    pub pc_offset: EmulatorCell,        // PCoffset9 (sign-extended)
    pub pointer_address: EmulatorCell,  // Address containing the final address
    pub indirect_address: EmulatorCell, // The final address loaded from pointer_address (set in execute)
    pub value_to_store: EmulatorCell,   // Value from SR to be stored (set in execute)
    pub is_valid_read_step1: bool,      // Flag if pointer_address is valid to read from
    pub is_valid_write_step2: bool, // Flag if indirect_address is valid to write to (set in execute)
}

impl Op for StiOp {
    fn decode(ir: EmulatorCell) -> Self {
        // LAYOUT: 1011 | SR | PCoffset9

        // Extract and sign-extend PCoffset9
        let pc_offset = ir.range(8..0).sext(8);

        Self {
            sr: ir.range(11..9),
            pc_offset,
            pointer_address: EmulatorCell::new(0),
            indirect_address: EmulatorCell::new(0),
            value_to_store: EmulatorCell::new(0),
            is_valid_read_step1: false,
            is_valid_write_step2: false,
        }
    }

    fn evaluate_address(&mut self, machine_state: &mut Emulator) {
        // Calculate the address of the pointer word.
        let current_pc = machine_state.pc;
        let pointer_addr_val = current_pc.get().wrapping_add(self.pc_offset.get());
        self.pointer_address.set(pointer_addr_val);

        // Check memory read permissions for the pointer address.
        let pointer_area = area_from_address(&self.pointer_address);
        if pointer_area.can_read(&machine_state.priv_level()) {
            self.is_valid_read_step1 = true;
        } else {
            // Privilege violation: Cannot read the pointer address.
            machine_state.exception = Some(Exception::new_access_control_violation());
            self.is_valid_read_step1 = false;
            tracing::warn!(
                address = format!("0x{:04X}", self.pointer_address.get()),
                "STI Privilege Violation (Step 1): Cannot read pointer address"
            );
        }
    }

    fn fetch_operands(&mut self, machine_state: &mut Emulator) -> bool {
        if self.is_valid_read_step1 {
            // Set MAR to fetch the pointer value (the indirect address) from memory.
            machine_state.mar = self.pointer_address;
        }
        false
    }

    fn execute_operation(&mut self, machine_state: &mut Emulator) {
        // This phase occurs after the memory read triggered by fetch_operands.
        // MDR now holds the indirect address read from Mem[pointer_address].
        if !self.is_valid_read_step1 {
            return; // Skip if the first read was invalid.
        }

        // Store the indirect address read from MDR.
        self.indirect_address = machine_state.mdr;

        // Check write permissions for the final indirect address.
        let indirect_area = area_from_address(&self.indirect_address);
        if indirect_area.can_write(&machine_state.priv_level()) {
            self.is_valid_write_step2 = true;
            // Fetch the value to store from the source register (SR).
            self.value_to_store = machine_state.r[self.sr.get() as usize];
        } else {
            // Privilege violation: Cannot write to the final indirect address.
            machine_state.exception = Some(Exception::new_access_control_violation());
            self.is_valid_write_step2 = false;
            tracing::warn!(
                address = format!("0x{:04X}", self.indirect_address.get()),
                "STI Privilege Violation (Step 2): Cannot write to final indirect address"
            );
        }
    }

    fn store_result(&mut self, machine_state: &mut Emulator) {
        // Set up MAR and MDR for the final memory write if all checks passed.
        if self.is_valid_read_step1 && self.is_valid_write_step2 {
            // Set MAR to the final destination address (indirect address).
            machine_state.mar = self.indirect_address;
            // Set MDR to the value fetched from SR during execute_operation.
            machine_state.mdr = self.value_to_store;
            // Signal the main loop to perform the memory write (Mem[MAR] <- MDR).
            machine_state.write_bit = true;
        }
    }
}
use std::fmt;

impl fmt::Display for StiOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display based on the state after decode (sr and pc_offset are known)
        // indirect_address, value_to_store, etc., are determined later
        // and are not part of the basic instruction format.

        let sr_index = self.sr.get();
        // Sign-extend the 9-bit offset for correct decimal representation
        let offset_val = self.pc_offset.sext(8).get() as i16;

        write!(
            f,
            "STI R{}, #{} (x{:03X})",
            sr_index,
            offset_val,                   // Display signed decimal offset
            self.pc_offset.get() & 0x1FF  // Display raw 9-bit hex offset
        )?;

        if self.is_valid_read_step1 && self.is_valid_write_step2 {
            write!(
                f,
                " [storing {:04X} to mem[mem[{:04X}]] = mem[{:04X}]]",
                self.value_to_store.get(),
                self.pointer_address.get(),
                self.indirect_address.get()
            )?;
        }
        Ok(())
    }
}
