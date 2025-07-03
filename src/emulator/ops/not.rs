use crate::emulator::{AluOp, BitAddressable, Emulator, EmulatorCell};

use super::Op;

#[derive(Debug, Clone)]
/// Preform the alu Not op on some register then save the result into some other register
pub enum NotOp {
    Decoded {
        dr: EmulatorCell, // Destination register index
        sr: EmulatorCell, // Source register index
    },
    Ready {
        dr: EmulatorCell, // Destination register index
        op: EmulatorCell, // Fetched source operand
    },
}

impl Op for NotOp {
    fn decode(ir: EmulatorCell) -> Self {
        // LAYOUT: 1001 | DR | SR | 111111
        let dr = ir.range(11..9);
        let sr = ir.range(8..6);

        NotOp::Decoded { dr, sr }
    }

    fn fetch_operands(&mut self, machine_state: &mut Emulator) -> bool {
        let mut new_op = None;
        if let NotOp::Decoded { dr, sr } = *self {
            let op = machine_state.r[sr.get() as usize];
            new_op = Some(NotOp::Ready { dr, op });
        } else {
            // Should not be in Ready state when fetch_operands is called again
            tracing::warn!("NOT: fetch_operands called when already in Ready state");
            debug_assert!(false, "Unexpected state flow in NOT fetch_operands");
        }

        if let Some(op) = new_op {
            *self = op;
        }
        // No second fetch phase needed for NOT.
        false
    }

    fn execute_operation(&mut self, machine_state: &mut Emulator) {
        if let NotOp::Ready { op, .. } = *self {
            // Set the ALU operation to NOT with the fetched operand
            machine_state.alu.op = Some(AluOp::Not(op));
        } else {
            // Should be in the Ready state by now
            tracing::warn!("NOT: execute_operation called before operands were fetched");
            debug_assert!(false, "NOT execute_operation called in unexpected state");
        }
    }

    fn store_result(&mut self, machine_state: &mut Emulator) {
        if let NotOp::Ready { dr, .. } = *self {
            // Result is available in alu_out after the ALU cycle completes implicitly
            let result = machine_state.alu.alu_out;
            let dr_idx = dr.get() as usize;

            // Store the result in the destination register
            machine_state.r[dr_idx] = result;

            // Update condition codes based on the result stored in DR
            machine_state.update_flags(dr_idx);
        } else {
            // Should be in the Ready state by now
            tracing::warn!("NOT: store_result called before operands were fetched");
            debug_assert!(false, "NOT store_result called in unexpected state");
        }
    }
}
use std::fmt;

impl fmt::Display for NotOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotOp::Decoded { dr, sr } => {
                write!(f, "NOT R{}, R{}", dr.get(), sr.get())
            }
            NotOp::Ready { dr, op } => {
                write!(f, "not R{}, x{:04X}", dr.get(), op.get())
            }
        }
    }
}
