use crate::emulator::{
    area_from_address, BitAddressable, Emulator, EmulatorCell, Exception, PSR_ADDR,
};

use super::Op;

#[derive(Debug, Clone)]
pub struct StOp {
    pub sr: EmulatorCell,                // Source Register index
    pub pc_offset: EmulatorCell,         // PCoffset9 (sign-extended)
    pub effective_address: EmulatorCell, // Calculated address
    pub is_valid_store: bool,            // Flag if the address is valid to write to
}

impl Op for StOp {
    fn decode(ir: EmulatorCell) -> Self {
        // LAYOUT: 0011 | SR | PCoffset9
        let sr = ir.range(11..9);
        let pc_offset = ir.range(8..0).sext(8);

        Self {
            sr,
            pc_offset,
            effective_address: EmulatorCell::new(0),
            is_valid_store: false,
        }
    }

    fn evaluate_address(&mut self, machine_state: &mut Emulator) {
        // Calculate effective address: PC + SEXT(PCoffset9)
        // PC was already incremented during the fetch phase
        let current_pc = machine_state.pc;
        let effective_addr_val = current_pc.get().wrapping_add(self.pc_offset.get());
        self.effective_address.set(effective_addr_val);

        // Check memory write permissions
        let target_area = area_from_address(&self.effective_address);
        if target_area.can_write(&machine_state.priv_level()) {
            self.is_valid_store = true;
        } else {
            // Privilege violation: Cannot write to this memory location
            machine_state.exception = Some(Exception::new_access_control_violation());
            self.is_valid_store = false;
            tracing::warn!(
                address = format!("0x{:04X}", self.effective_address.get()),
                "ST Privilege Violation: Cannot write to address"
            );
        }
    }

    fn store_result(&mut self, machine_state: &mut Emulator) {
        if self.is_valid_store {
            // Set MAR to the target address
            machine_state.mar = self.effective_address;
            // Fetch the value from the source register into MDR
            let sr_index = self.sr.get() as usize;
            machine_state.mdr = machine_state.r[sr_index];
            // we are writing mem[mar] <- mdr
            machine_state.write_bit = true;
        }
        if machine_state.mar.get() == PSR_ADDR as u16 {
            let new_psr = machine_state.mdr;
            machine_state.memory[PSR_ADDR].set(new_psr.get());
        }
    }
}
use std::fmt;

impl fmt::Display for StOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sr_index = self.sr.get();
        // pc_offset is already sign-extended during decode.
        let offset_val = self.pc_offset.get() as i16; // Cast to signed for display

        write!(
            f,
            "ST R{}, #{} (x{:03X})",
            sr_index,
            offset_val,
            self.pc_offset.get() & 0x1FF // Mask to 9 bits for hex
        )
    }
}
