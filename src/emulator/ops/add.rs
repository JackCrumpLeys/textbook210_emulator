use tracing_subscriber::filter::combinator::And;

use crate::emulator::{AluOp, BitAddressable, Emulator, EmulatorCell};
use std::fmt;

use super::Op;

#[derive(Debug, Clone)]
pub enum AddOp {
    Immidiate {
        dr: EmulatorCell,
        sr1: EmulatorCell,
        imm5: EmulatorCell,
    },
    Register {
        dr: EmulatorCell,
        sr1: EmulatorCell,
        sr2: EmulatorCell,
    },
    Ready {
        dr: EmulatorCell,
        op1: EmulatorCell,
        op2: EmulatorCell,
    },
}

impl Op for AddOp {
    fn decode(ir: EmulatorCell) -> Self {
        let span = tracing::trace_span!("ADD_decode", op = ir.get());
        let _enter = span.enter();

        // LAYOUT: 0001 | DR | SR1 | ImmTBit | (Register || Immediate)
        let dr = ir.range(11..9);
        let sr1 = ir.range(8..6);

        // Check immediate mode (bit[5])
        match ir.index(5).get() {
            0 => {
                // Register mode
                let sr2 = ir.range(2..0);
                AddOp::Register { dr, sr1, sr2 }
            }
            1 => {
                // Immediate mode
                // imm5 is bits 4..0
                let imm5 = ir.range(4..0);

                AddOp::Immidiate { dr, sr1, imm5 }
            }
            _ => unreachable!("Bit 5 can only be 0 or 1"),
        }
    }
    // Fetch Operands: Get values from registers/immediate and store temporarily in Emulator state.
    fn fetch_operands(&mut self, machine_state: &mut Emulator) -> bool {
        let span = tracing::trace_span!("ADD_fetch_operands", op = ?self);
        let _enter = span.enter();

        let mut new_op = None;

        match *self {
            AddOp::Register { dr, sr1, sr2 } => {
                let op1 = machine_state.r[sr1.get() as usize];
                let op2 = machine_state.r[sr2.get() as usize];
                // Transition to the Ready state with fetched operands
                new_op = Some(AddOp::Ready { dr, op1, op2 });
            }
            AddOp::Immidiate { dr, sr1, imm5 } => {
                let op1 = machine_state.r[sr1.get() as usize];
                // Sign extend imm5 (5 bits) using the BitAddressable helper method
                let op2 = imm5.sext(4);
                // Transition to the Ready state with fetched operands
                new_op = Some(AddOp::Ready { dr, op1, op2 });
            }
            AddOp::Ready { .. } => {
                // This state implies operands might have been fetched differently,
                // but based on the typical instruction cycle, fetch_operands
                // shouldn't encounter this state if called correctly.
                // If using temporary Emulator state, this branch is likely unreachable.
                tracing::warn!("ADD: Encountered Ready state during fetch_operands phase. This might indicate unexpected state flow.");
                debug_assert!(false, "Unexpected state flow");
                // Depending on design, might need to extract from self here,
                // but current implementation uses Emulator temp state.
            }
        }
        if let Some(op) = new_op {
            *self = op;
        }

        false
    }

    // Execute: Perform the addition using the fetched operands.
    fn execute_operation(&mut self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("ADD_execute", op = ?self);
        let _enter = span.enter();

        if let AddOp::Ready { op1, op2, .. } = self {
            machine_state.alu.op = Some(AluOp::Add(*op1, *op2)); // handled between states
        } else {
            // Should be in the Ready state by now
            debug_assert!(
                false,
                "ADD execute_operation called before operands were fetched (not in Ready state)"
            );
        }
    }

    // Store Result: Write the result from the ALU back to the destination register and update flags.
    fn store_result(&mut self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("ADD_store_result", op = ?self);
        let _enter = span.enter();

        if let AddOp::Ready { dr, .. } = self {
            let result = machine_state.alu.alu_out.get();
            let dr_idx = dr.get() as usize;
            machine_state.r[dr_idx].set(result);
            machine_state.update_flags(dr_idx);
        } else {
            // Should ideally be in the Ready state by now
            debug_assert!(
                false,
                "ADD store_result called before operands were fetched (not in Ready state)"
            );
        }
    }
}

impl fmt::Display for AddOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddOp::Immidiate { dr, sr1, imm5 } => {
                // Sign extend imm5 for correct decimal representation
                let imm_value = imm5.sext(4).get() as i16; // sext from bit 4 for 5 bits
                write!(
                    f,
                    "ADD R{}, R{}, #{:?} (x{:02X})",
                    dr.get(),
                    sr1.get(),
                    imm_value,
                    imm5.get() & 0x1F // Mask to 5 bits for hex
                )
            }
            AddOp::Register { dr, sr1, sr2 } => {
                write!(f, "ADD R{}, R{}, R{}", dr.get(), sr1.get(), sr2.get())
            }
            AddOp::Ready { .. } => {
                // This state represents an internal step, not a directly displayable instruction
                write!(f, "INVALID READ")
            }
        }
    }
}
