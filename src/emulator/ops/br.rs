use crate::emulator::micro_op::{CycleState, MicroOp, MicroOpGenerator};
use crate::emulator::{area_from_address, BitAddressable, Emulator, EmulatorCell, Exception};
use crate::micro_op;
use std::collections::HashMap;

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

impl MicroOpGenerator for BrOp {
    fn generate_plan(&self) -> HashMap<CycleState, Vec<MicroOp>> {
        let mut plan = HashMap::new();
        // The branch condition depends on the processor status register (PSR), which is only known at execution time.
        // Therefore, we calculate the potential address first, then use a custom micro-op to conditionally jump.

        // Evaluate Address Phase: Calculate the potential target address.
        plan.insert(
            CycleState::EvaluateAddress,
            vec![micro_op!(ALU_OUT <- PC + PCOFFSET(self.pc_offset.get() as i16))],
        );

        // Execute Phase: Use a custom micro-op to check conditions and update PC if necessary.
        let n_cond = self.n_bit.get() == 1;
        let z_cond = self.z_bit.get() == 1;
        let p_cond = self.p_bit.get() == 1;

        let mut if_cond = vec![];

        if n_cond {
            if_cond.push("N == 1");
        }
        if z_cond {
            if_cond.push("Z == 1");
        }
        if p_cond {
            if_cond.push("P == 1");
        }
        let empty = if_cond.is_empty();
        let if_cond = if_cond.join(" OR ");

        let psudo_str = if empty {
            "; for BR (No conditions) jump never triggers and always just go forward one step"
                .to_string()
        } else {
            format!(
                "IF {if_cond}
                        ALU_OUT <- PC + SEXT(PCoffset9)
                        PC <- ALU_OUT"
            )
        };

        plan.insert(
            CycleState::Execute,
            vec![MicroOp::new_custom(
                move |emu: &mut Emulator| {
                    let (psr_n, psr_z, psr_p) = emu.get_nzp();
                    let branch_taken = (n_cond && psr_n) || (z_cond && psr_z) || (p_cond && psr_p);

                    if branch_taken {
                        // Branch is taken. The target address is in ALU_OUT.
                        let target_address = emu.alu.alu_out;

                        // Before jumping, check for memory access violations.
                        let target_area = area_from_address(&target_address);
                        if target_area.can_read(&emu.priv_level()) {
                            emu.pc.set(target_address.get());
                        } else {
                            return Err(Exception::AccessControlViolation);
                        }
                    }
                    // If branch is not taken, do nothing.
                    Ok(())
                },
                psudo_str,
            )],
        );

        plan
    }
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
