use crate::emulator::micro_op::{CycleState, MicroOp, MicroOpGenerator};
use crate::emulator::{BitAddressable, EmulatorCell};
use crate::micro_op;
use std::collections::HashMap;

use super::Op;

#[derive(Debug, Clone)]
/// Are we looking at jsr or jsrr
pub enum JsrMode {
    /// JSR: jump to a sub-routine the adress at pc + imm11
    Relative { pc_offset: EmulatorCell },
    /// JSRR: jump to a sub-routine the adress stored at a given register
    Register { base_r: EmulatorCell },
}

#[derive(Debug, Clone)]
/// Jump to a sub routine either directly or via pc offset
pub struct JsrOp {
    /// are we looking at jst or jsrr?
    pub mode: JsrMode,
    /// Where we boutta jump?
    pub target_address: EmulatorCell, // Calculated during evaluate_address
    /// Can we jump to teh place we going?
    pub is_valid_jump: bool, // Set during evaluate_address
}

impl MicroOpGenerator for JsrOp {
    fn generate_plan(&self) -> HashMap<CycleState, Vec<MicroOp>> {
        let mut plan = HashMap::new();

        // Evaluate Address phase - save return address and calculate target
        plan.insert(
            CycleState::EvaluateAddress,
            vec![
                micro_op!(R(7) <- PC), // Save return address in R7
                match &self.mode {
                    JsrMode::Relative { pc_offset } => {
                        micro_op!(ALU_OUT <- PC + PCOFFSET(pc_offset.get() as i16))
                    }
                    JsrMode::Register { base_r } => {
                        micro_op!(Temp <- R(base_r.get()))
                    }
                },
            ],
        );

        // Execute phase - jump to target address
        plan.insert(
            CycleState::Execute,
            vec![
                micro_op!(MSG format!("Jump to subroutine at target address")),
                match &self.mode {
                    JsrMode::Relative { .. } => micro_op!(PC <- AluOut),
                    JsrMode::Register { .. } => micro_op!(PC <- Temp),
                },
            ],
        );

        plan
    }
}

impl Op for JsrOp {
    fn decode(ir: EmulatorCell) -> Self {
        // LAYOUT: 0100 | ToggleBit | rest of bits (PCoffset11 or BaseR)

        let mode = if ir.index(11).get() == 1 {
            // JSR: Use PC-relative addressing
            // Extract and sign-extend PCoffset11
            let pc_offset = ir.range(10..0).sext(10);
            JsrMode::Relative { pc_offset }
        } else {
            // JSRR: Get address from base register
            let base_r = ir.range(8..6);
            JsrMode::Register { base_r }
        };

        Self {
            mode,
            target_address: EmulatorCell::new(0),
            is_valid_jump: false,
        }
    }
}

use std::fmt;

impl fmt::Display for JsrOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display implementation based on the state after decode.
        match &self.mode {
            JsrMode::Relative { pc_offset } => {
                // JSR: Display with PC-relative offset
                let offset_val = pc_offset.get() as i16; // Cast to signed for decimal display
                write!(
                    f,
                    "JSR #{} (x{:03X})",
                    offset_val,
                    pc_offset.get() & 0x7FF // Mask to 11 bits for hex
                )?;
            }
            JsrMode::Register { base_r } => {
                // JSRR: Display with base register
                write!(f, "JSRR R{}", base_r.get())?;
            }
        }
        if self.is_valid_jump {
            write!(f, " [jumping")?;
            if self.target_address.get() != 0 {
                write!(f, " to x{:04X}", self.target_address.get())?;
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}
