use crate::emulator::{area_from_address, BitAddressable, Emulator, EmulatorCell, Exception};

use super::Op;

#[derive(Debug, Clone)]
pub struct BrOp {
    n_bit: EmulatorCell,
    z_bit: EmulatorCell,
    p_bit: EmulatorCell,
    pc_offset: EmulatorCell,
    branch_taken: bool,           // Set during evaluate_address
    target_address: EmulatorCell, // Set during evaluate_address if branch_taken is true
}

impl Op for BrOp {
    fn decode(ir: EmulatorCell) -> Self {
        // LAYOUT: 0000 | N | Z | P | PCoffset9

        // Extract NZP bits and PCoffset9
        let mut n_bit = ir.index(11);
        let mut z_bit = ir.index(10);
        let mut p_bit = ir.index(9);

        if ir.range(11..9).get() == 0 {
            n_bit = EmulatorCell::new(1);
            z_bit = EmulatorCell::new(1);
            p_bit = EmulatorCell::new(1);
        }

        // Extract and sign-extend PCoffset9
        let pc_offset = ir.range(8..0).sext(8);

        Self {
            n_bit,
            z_bit,
            p_bit,
            pc_offset,
            branch_taken: false,
            target_address: EmulatorCell::new(0),
        }
    }

    fn evaluate_address(&mut self, machine_state: &mut Emulator) {
        // Check if condition codes match current state
        let n_match = self.n_bit.get() == 1 && machine_state.n.get() == 1;
        let z_match = self.z_bit.get() == 1 && machine_state.z.get() == 1;
        let p_match = self.p_bit.get() == 1 && machine_state.p.get() == 1;

        // If any condition matches, calculate the target address and mark branch as taken
        if n_match || z_match || p_match {
            // PC has already been incremented in fetch, so we add the offset directly
            let current_pc = machine_state.pc.get();
            // Note: pc_offset is already sign-extended
            let new_pc_val = current_pc.wrapping_add(self.pc_offset.get());
            self.target_address.set(new_pc_val);
            self.branch_taken = true;
        } else {
            self.branch_taken = false;
        }
    }

    fn execute_operation(&mut self, machine_state: &mut Emulator) {
        if self.branch_taken {
            // Check memory permissions before jumping
            let target_area = area_from_address(&self.target_address);
            if target_area.can_read(&machine_state.current_privilege_level) {
                machine_state.pc.set(self.target_address.get());
            } else {
                // Cannot jump to non-readable memory
                machine_state.exception = Some(Exception::new_access_control_violation());
            }
        }
        // If branch_taken is false, PC remains as incremented in fetch phase.
    }
}

impl std::fmt::Display for BrOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut op_name = "BR".to_string();
        if self.n_bit.get() == 1 {
            op_name.push('N');
        }
        if self.z_bit.get() == 1 {
            op_name.push('Z');
        }
        if self.p_bit.get() == 1 {
            op_name.push('P');
        }
        // If no flags are set, it's technically BR (unconditional)
        if op_name == "BR" {
            op_name.push_str("nzp"); // Or just "BR" depending on convention
        }

        // Format offset as signed decimal
        let offset_val = self.pc_offset.get() as i16; // Cast to signed for proper display

        write!(
            f,
            "{} #{} (x{:03X})",
            op_name,
            offset_val,
            self.pc_offset.get() & 0x1FF
        )
    }
}
