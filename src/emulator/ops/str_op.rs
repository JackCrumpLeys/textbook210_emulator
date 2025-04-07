use crate::emulator::{area_from_address, BitAddressable, Emulator, EmulatorCell, Exception};

use super::Op;

#[derive(Debug, Clone)]
pub struct StrOp {
    sr: EmulatorCell,                // Source Register index
    base_r: EmulatorCell,            // Base Register index
    offset6: EmulatorCell,           // offset6 (sign-extended)
    effective_address: EmulatorCell, // Calculated address
    value_to_store: EmulatorCell,    // Value from SR (fetched in fetch_operands)
    is_valid_store: bool,            // Flag if the address is valid to write to
}

impl Op for StrOp {
    fn decode(ir: EmulatorCell) -> Self {
        // LAYOUT: 0111 | SR | BaseR | offset6
        let sr = ir.range(11..9);
        let base_r = ir.range(8..6);
        // Extract and sign-extend offset6
        let offset6 = ir.range(5..0).sext(5);

        Self {
            sr,
            base_r,
            offset6,
            effective_address: EmulatorCell::new(0),
            value_to_store: EmulatorCell::new(0),
            is_valid_store: false,
        }
    }

    fn evaluate_address(&mut self, machine_state: &mut Emulator) {
        // Calculate effective address: BaseR + SEXT(offset6)
        let base_r_value = machine_state.r[self.base_r.get() as usize];
        let effective_addr_val = base_r_value.get().wrapping_add(self.offset6.get());
        self.effective_address.set(effective_addr_val);

        // Check memory write permissions
        let target_area = area_from_address(&self.effective_address);
        if target_area.can_write(&machine_state.current_privilege_level) {
            self.is_valid_store = true;
        } else {
            // Privilege violation: Cannot write to this memory location
            machine_state.exception = Some(Exception::new_access_control_violation());
            self.is_valid_store = false;
            tracing::warn!(
                address = format!("0x{:04X}", self.effective_address.get()),
                "STR Privilege Violation: Cannot write to address"
            );
        }
    }

    fn fetch_operands(&mut self, machine_state: &mut Emulator) -> bool {
        // Fetch the value from the source register (SR) if the store address is valid.
        if self.is_valid_store {
            self.value_to_store = machine_state.r[self.sr.get() as usize];
        }
        false
    }

    fn store_result(&mut self, machine_state: &mut Emulator) {
        // Set up MAR and MDR for the memory write if the store is valid.
        if self.is_valid_store {
            // Set MAR to the final destination address.
            machine_state.mar = self.effective_address;
            // Set MDR to the value fetched from SR during fetch_operands.
            machine_state.mdr = self.value_to_store;
            // Signal the main loop to perform the memory write (Mem[MAR] <- MDR).
            machine_state.write_bit = true;
        }
        // If !is_valid_store, an exception was set, and the store is skipped.
    }
}
use std::fmt;

impl fmt::Display for StrOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Get the raw 6-bit offset value
        let offset_val_6bit = self.offset6.get() & 0x3F;
        // Calculate the sign-extended value (as i16 for display)
        let offset_val_sext = if (offset_val_6bit >> 5) & 1 == 1 {
            // Negative number, extend with 1s
            (offset_val_6bit as i16) | !0x3F // or (offset_val_6bit as i16) - 64
        } else {
            offset_val_6bit as i16
        };

        write!(
            f,
            "STR R{}, R{}, #{} (x{:02X})",
            self.sr.get(),
            self.base_r.get(),
            offset_val_sext, // Display sign-extended decimal
            offset_val_6bit  // Display raw 6-bit hex
        )
    }
}
