use crate::emulator::micro_op::{CycleState, MicroOp, MicroOpGenerator};
use crate::emulator::{BitAddressable, EmulatorCell};
use crate::micro_op;
use std::collections::HashMap;

use super::Op;

#[derive(Debug, Clone)]
/// Load from some ofset of PC
pub struct LdOp {
    /// Where do we Store the result of the load
    pub dr: EmulatorCell, // Destination Register index
    /// What to ofset our pc by to get the value
    pub pc_offset: EmulatorCell, // PCoffset9 (sign-extended)
    /// In the end where are we loading from
    pub effective_address: EmulatorCell, // Calculated address
    /// Are we allowed to load from this location
    pub is_valid_load: bool, // Flag if the address is valid to read from
}

impl MicroOpGenerator for LdOp {
    fn generate_plan(&self) -> HashMap<CycleState, Vec<MicroOp>> {
        let mut plan = HashMap::new();

        // Evaluate Address phase - calculate effective address
        plan.insert(
            CycleState::EvaluateAddress,
            vec![micro_op!(ALU_OUT <- PC + PCOFFSET(self.pc_offset.get() as i16))],
        );

        plan.insert(CycleState::FetchOperands, vec![micro_op!(MAR <- AluOut)]);
        // Memory read MDR <- MEM[MAR] happens implicitly between phases

        // Store Result phase - move loaded value to destination register
        plan.insert(
            CycleState::StoreResult,
            vec![
                micro_op!(R(self.dr.get()) <- MDR),
                micro_op!(SET_CC(self.dr.get())),
            ],
        );

        plan
    }
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
}

use std::fmt;

impl fmt::Display for LdOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display the state immediately after decode.
        // effective_address and is_valid_load are calculated later.
        let dr_index = self.dr.get();
        let offset_val = self.pc_offset.get() as i16; // Cast to signed for display
        let offset_hex = self.pc_offset.get() & 0x1FF; // Mask to 9 bits for hex

        write!(f, "LD R{dr_index}, #{offset_val} (x{offset_hex:03X})")?;

        if self.is_valid_load {
            write!(f, " [loading")?;
            if self.effective_address.get() != 0 {
                write!(f, " from x{:04X}", self.effective_address.get())?;
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}
