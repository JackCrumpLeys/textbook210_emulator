use crate::emulator::{BitAddressable, Emulator, EmulatorCell};

use super::Op;

#[derive(Debug, Clone)]
pub struct LeaOp {
    dr: EmulatorCell,                // Destination Register index
    pc_offset: EmulatorCell,         // PCoffset9 (sign-extended)
    effective_address: EmulatorCell, // Calculated address
}

impl Op for LeaOp {
    fn decode(ir: EmulatorCell) -> Self {
        // LAYOUT: 1110 | DR | PCoffset9
        let dr = ir.range(11..9);
        // Extract and sign-extend PCoffset9 during decode
        let pc_offset = ir.range(8..0).sext(8);

        Self {
            dr,
            pc_offset,
            effective_address: EmulatorCell::new(0), // Initialize
        }
    }

    fn evaluate_address(&mut self, machine_state: &mut Emulator) {
        // Calculate effective address: PC + SEXT(PCoffset9)
        let current_pc = machine_state.pc;
        let effective_addr_val = current_pc.get().wrapping_add(self.pc_offset.get());
        self.effective_address.set(effective_addr_val);
    }

    fn fetch_operands(&mut self, _machine_state: &mut Emulator) -> bool {
        // LEA does not fetch operands from memory or registers based on the address.
        false
    }

    fn store_result(&mut self, machine_state: &mut Emulator) {
        // Load the calculated effective address into the destination register.
        let dr_index = self.dr.get() as usize;
        machine_state.r[dr_index] = self.effective_address;
    }
}

impl std::fmt::Display for LeaOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dr_index = self.dr.get();
        let offset_val = self.pc_offset.get() as i16; // Cast to signed for proper display

        write!(
            f,
            "LEA R{}, #{} (x{:03X})",
            dr_index,
            offset_val,
            self.pc_offset.get() & 0x1FF
        )
    }
}
