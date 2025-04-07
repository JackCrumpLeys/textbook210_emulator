use crate::emulator::{BitAddressable, Emulator};

use crate::emulator::{AluOp, EmulatorCell};

use super::Op;

#[derive(Debug, Clone)]
pub enum AndOp {
    Immediate {
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

impl Op for AndOp {
    fn decode(ir: EmulatorCell) -> Self {
        let span = tracing::trace_span!("AND_decode", op = ir.get());
        let _enter = span.enter();

        // LAYOUT: 0101 | DR | SR1 | ImmTBit | (Register || Immediate)
        let dr = ir.range(11..9);
        let sr1 = ir.range(8..6);

        // Check immediate mode (bit[5])
        match ir.index(5).get() {
            0 => {
                // Register mode
                let sr2 = ir.range(2..0);
                AndOp::Register { dr, sr1, sr2 }
            }
            1 => {
                // Immediate mode
                // imm5 is bits 4..0
                let imm5 = ir.range(4..0);
                AndOp::Immediate { dr, sr1, imm5 }
            }
            _ => unreachable!("Bit 5 can only be 0 or 1"),
        }
    }

    fn fetch_operands(&mut self, machine_state: &mut Emulator) -> bool {
        let span = tracing::trace_span!("AND_fetch_operands", op = ?self);
        let _enter = span.enter();

        let mut new_op = None;

        match *self {
            AndOp::Register { dr, sr1, sr2 } => {
                let op1 = machine_state.r[sr1.get() as usize];
                let op2 = machine_state.r[sr2.get() as usize];
                new_op = Some(AndOp::Ready { dr, op1, op2 });
            }
            AndOp::Immediate { dr, sr1, imm5 } => {
                let op1 = machine_state.r[sr1.get() as usize];
                // Sign extend imm5 (5 bits) using the BitAddressable helper method
                let op2 = imm5.sext(4);
                new_op = Some(AndOp::Ready { dr, op1, op2 });
            }
            AndOp::Ready { .. } => {
                tracing::error!("AND: Encountered Ready state during fetch_operands phase. This might indicate unexpected state flow.");
                debug_assert!(false, "Unexpected state flow");
            }
        }
        if let Some(op) = new_op {
            *self = op;
        }

        false
    }

    fn execute_operation(&mut self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("AND_execute_operation", op = ?self);
        let _enter = span.enter();

        if let AndOp::Ready { op1, op2, .. } = self {
            machine_state.alu.op = Some(AluOp::And(*op1, *op2));
        } else {
            debug_assert!(
                false,
                "AND execute_operation called before operands were fetched (not in Ready state)"
            );
        }
    }

    fn store_result(&mut self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("AND_store_result", op = ?self);
        let _enter = span.enter();

        if let AndOp::Ready { dr, .. } = self {
            let result = machine_state.alu.alu_out.get();

            let dr_idx = dr.get() as usize;
            machine_state.r[dr_idx].set(result);
            machine_state.update_flags(dr_idx);
        } else {
            debug_assert!(
                false,
                "AND store_result called before operands were fetched (not in Ready state)"
            );
        }
    }
}

use std::fmt;

impl fmt::Display for AndOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AndOp::Register { dr, sr1, sr2 } => {
                write!(f, "AND R{}, R{}, R{}", dr.get(), sr1.get(), sr2.get())
            }
            AndOp::Immediate { dr, sr1, imm5 } => {
                // Get the raw 5-bit value
                let imm_val_5bit = imm5.get() & 0x1F;
                // Calculate the sign-extended value (as i16 for display)
                let imm_val_sext = if (imm_val_5bit >> 4) & 1 == 1 {
                    // Negative number, extend with 1s
                    (imm_val_5bit as i16) | !0x1F // or (imm_val_5bit as i16) - 32
                } else {
                    imm_val_5bit as i16
                };
                write!(
                    f,
                    "AND R{}, R{}, #{} (x{:02X})",
                    dr.get(),
                    sr1.get(),
                    imm_val_sext, // Display sign-extended decimal
                    imm_val_5bit  // Display raw 5-bit hex
                )
            }
            AndOp::Ready { .. } => {
                write!(f, "INVALID READ") // Cannot display from Ready state
            }
        }
    }
}
