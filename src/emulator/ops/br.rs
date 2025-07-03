use crate::emulator::{area_from_address, BitAddressable, Emulator, EmulatorCell, Exception};

use super::Op;

#[derive(Debug, Clone)]

pub struct BrOp {
    /// Do we match on negitive?
    pub n_bit: EmulatorCell,
    /// Do we match on zero?
    pub z_bit: EmulatorCell,
    /// Do we match on positive?
    pub p_bit: EmulatorCell,
    /// Offset from pc to take if we match
    pub pc_offset: EmulatorCell,
    /// Do the condition codes match and we are going to branch?
    pub branch_taken: bool, // Set during evaluate_address
    /// Where we boutta go
    pub target_address: EmulatorCell, // Set during evaluate_address if branch_taken is true
}

impl Op for BrOp {
    fn decode(ir: EmulatorCell) -> Self {
        // LAYOUT: 0000 | N | Z | P | PCoffset9

        // Extract and sign-extend PCoffset9
        let pc_offset = ir.range(8..0).sext(8);

        Self {
            n_bit: ir.index(11),
            z_bit: ir.index(10),
            p_bit: ir.index(9),
            pc_offset,
            branch_taken: false,
            target_address: EmulatorCell::new(0),
        }
    }

    fn evaluate_address(&mut self, machine_state: &mut Emulator) {
        // Check if condition codes match current state
        let (n, z, p) = machine_state.get_nzp();
        let n_match = self.n_bit.get() == 1 && n;
        let z_match = self.z_bit.get() == 1 && z;
        let p_match = self.p_bit.get() == 1 && p;

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
            if target_area.can_read(&machine_state.priv_level()) {
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

        // Format offset as signed decimal
        let offset_val = self.pc_offset.get() as i16; // Cast to signed for proper display

        write!(
            f,
            "{} #{} (x{:03X})",
            op_name,
            offset_val,
            self.pc_offset.get() & 0x1FF
        )?;

        if self.branch_taken {
            write!(f, " [branching")?;
            if self.target_address.get() != 0 {
                write!(f, " to x{:04X}", self.target_address.get())?;
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}
