use crate::emulator::micro_op::{CycleState, MicroOp, MicroOpGenerator};
use crate::emulator::{BitAddressable, EmulatorCell};
use crate::micro_op;
use std::collections::HashMap;

use super::Op;
#[derive(Debug, Clone)]
pub struct JmpOp {
    /// Base register index, this is added to the offset to calculate where to jump
    pub base_r: EmulatorCell,
    /// Where we boutta go
    pub target_address: EmulatorCell, // Calculated during evaluate_address
    /// Can we go there?
    pub is_valid_jump: bool, // Set during evaluate_address
}

impl MicroOpGenerator for JmpOp {
    fn generate_plan(&self) -> HashMap<CycleState, Vec<MicroOp>> {
        let mut plan = HashMap::new();

        // Evaluate Address phase - get target address from register
        plan.insert(
            CycleState::EvaluateAddress,
            vec![micro_op!(Temp <- R(self.base_r.get()))],
        );

        // Execute phase - update PC to target address
        plan.insert(
            CycleState::Execute,
            vec![
                micro_op!(MSG format!("Jump to address in R{}", self.base_r.get())),
                micro_op!(PC <- Temp),
            ],
        );

        plan
    }
}

impl Op for JmpOp {
    fn decode(ir: EmulatorCell) -> Self {
        // LAYOUT: 1100 | 000 | BaseR | 000000
        let base_r = ir.range(8..6);

        Self {
            base_r,
            target_address: EmulatorCell::new(0), // Initialize to 0
            is_valid_jump: false,                 // Initialize to false
        }
    }
}

use std::fmt;

impl fmt::Display for JmpOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let base_r_index = self.base_r.get();

        if base_r_index == 7 {
            write!(f, "RET")?;
        } else {
            write!(f, "JMP R{base_r_index}")?;
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
