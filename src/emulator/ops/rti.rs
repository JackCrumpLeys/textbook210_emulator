use crate::emulator::micro_op::{CycleState, MicroOp, MicroOpGenerator};
use crate::emulator::{BitAddressable, EmulatorCell, Exception, PSR_ADDR};
use crate::micro_op;
use std::collections::HashMap;

use super::Op;

#[derive(Debug, Clone)]
/// This is executed by the os on service routines to return control to the callee
pub struct RtiOp;

impl MicroOpGenerator for RtiOp {
    fn generate_plan(&self) -> HashMap<CycleState, Vec<MicroOp>> {
        let mut plan = HashMap::new();

        // Fetch Operands phase - read PC from stack
        plan.insert(
            CycleState::FetchOperands,
            vec![
                MicroOp::new_custom(
                    |emu| {
                        if emu.memory[PSR_ADDR].index(15).get() == 1 {
                            Err(Exception::PrivilegeViolation)
                        } else {
                            Ok(())
                        }
                    },
                    "
if (PSR[15] == 1)
    ; Initiate a privilege mode exception"
                        .to_owned(),
                ),
                micro_op!(MAR <- R(6)), // Set MAR to current SSP (R6)
                                        // First memory read happens implicitly: MDR <- MEM[MAR] (gets PC)
            ],
        );

        // Execute phase - save PC and read PSR
        plan.insert(
            CycleState::Execute,
            vec![
                micro_op!(Temp <- MDR),              // Save PC temporarily
                micro_op!(ALU_OUT <- R(6) + IMM(1)), // Calculate SSP + 1 for PSR read
                micro_op!(MAR <- AluOut),            // Set MAR to SSP + 1
                                                     // Second memory read happens implicitly: MDR <- MEM[MAR] (gets PSR)
            ],
        );

        // Store Result phase - restore state
        plan.insert(
            CycleState::StoreResult,
            vec![
                micro_op!(PC <- Temp),               // Restore PC
                micro_op!(PSR <- MDR),               // Restore PSR
                micro_op!(ALU_OUT <- R(6) + IMM(2)), // Calculate new SSP
                micro_op!(R(6) <- AluOut),           // Update SSP (R6 += 2)
                MicroOp::new_custom(
                    |emu| {
                        if emu.memory[PSR_ADDR].index(15).get() == 1 {
                            emu.saved_ssp = emu.r[6];
                            emu.r[6] = emu.saved_usp;
                        }
                        Ok(())
                    },
                    "\
if (PSR[15] == 1) {
    Saved_SSP <- R6
    R6 <- Saved_USP
}"
                    .to_owned(),
                ),
            ],
        );

        plan
    }
}

impl Op for RtiOp {
    fn decode(_ir: EmulatorCell) -> Self {
        // RTI has a fixed opcode (1000) and no operands in the instruction itself.
        Self
    }
}
use std::fmt;

impl fmt::Display for RtiOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // RTI has no operands encoded in the instruction to display.
        // The state fields (popped_pc, popped_psr, is_valid_rti) are determined
        // during execution, not decode, so they aren't part of the basic instruction display.
        write!(f, "RTI")
    }
}
