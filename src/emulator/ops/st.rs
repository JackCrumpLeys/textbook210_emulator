use crate::emulator::micro_op::{CycleState, MicroOp, MicroOpGenerator};
use crate::emulator::{BitAddressable, EmulatorCell};
use std::collections::HashMap;

use super::Op;

#[derive(Debug, Clone)]
pub struct StOp {
    pub sr: EmulatorCell,                // Source Register index
    pub pc_offset: EmulatorCell,         // PCoffset9 (sign-extended)
    pub effective_address: EmulatorCell, // Calculated address
    pub is_valid_store: bool,            // Flag if the address is valid to write to
}

impl MicroOpGenerator for StOp {
    fn generate_plan(&self) -> HashMap<CycleState, Vec<MicroOp>> {
        let mut plan = HashMap::new();

        // Evaluate Address phase - calculate effective address
        plan.insert(
            CycleState::EvaluateAddress,
            vec![micro_op!(ALU_OUT <- PC + PCOFFSET(self.pc_offset.get() as i16))],
        );

        // Store Result phase - trigger memory write
        plan.insert(
            CycleState::StoreResult,
            vec![
                micro_op!(MAR <- AluOut),
                micro_op!(MDR <- R(self.sr.get())),
                micro_op!(SET_FLAG(WriteMemory)),
            ],
        );

        plan
    }
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
        )?;

        if self.is_valid_store {
            write!(f, " [storing")?;
            if self.effective_address.get() != 0 {
                write!(f, " to x{:04X}", self.effective_address.get())?;
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}
