use crate::emulator::micro_op::{CycleState, MicroOp, MicroOpGenerator};
use crate::emulator::{
    area_from_address, BitAddressable, Emulator, EmulatorCell, Exception, PrivilegeLevel, PSR_ADDR,
};
use crate::micro_op;
use std::collections::HashMap;

use super::Op;

#[derive(Debug, Clone)]
/// This is executed by the os on service routines to return control to the callee
pub struct RtiOp {
    // No specific data needed during decode for RTI itself.
    // State needed for execution will be read directly from Emulator state.
    popped_pc: EmulatorCell,
    popped_psr: EmulatorCell,
    is_valid_rti: bool, // Flag set during evaluate_address if preconditions met
}

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
                    "if (PSR[15] == 1)
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
                        emu.saved_ssp = emu.r[6];
                        emu.r[6] = emu.saved_usp;
                        Ok(())
                    },
                    "Saved_SSP <- R6
                    R6 <- Saved_USP"
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
        Self {
            popped_pc: EmulatorCell::new(0),
            popped_psr: EmulatorCell::new(0),
            is_valid_rti: false, // Start assuming invalid until checked
        }
    }

    fn evaluate_address(&mut self, machine_state: &mut Emulator) {
        // Pre-check: RTI can only be executed in Supervisor mode.
        if matches!(machine_state.priv_level(), PrivilegeLevel::User) {
            machine_state.exception = Some(Exception::new_privilege_violation());
            self.is_valid_rti = false;
            tracing::warn!("RTI Privilege Violation: Attempted execution in User mode.");
            return; // Don't proceed further if in User mode
        }

        // In Supervisor mode, R6 is the Supervisor Stack Pointer (SSP).
        let ssp = machine_state.r[6];
        let pc_addr = ssp;
        let psr_addr = EmulatorCell::new(ssp.get().wrapping_add(1)); // Address after PC on stack

        // perm checks have no real reason to happen in this context but its good to be consistent when making a learning tool

        // Check if we can read PC from the stack
        let pc_area = area_from_address(&pc_addr);
        if !pc_area.can_read(&machine_state.priv_level()) {
            machine_state.exception = Some(Exception::new_access_control_violation());
            self.is_valid_rti = false;
            tracing::warn!(
                "RTI Access Violation: Cannot read PC from stack address 0x{:04X}",
                pc_addr.get()
            );
            return;
        }

        // Check if we can read PSR from the stack
        let psr_area = area_from_address(&psr_addr);
        if !psr_area.can_read(&machine_state.priv_level()) {
            machine_state.exception = Some(Exception::new_access_control_violation());
            self.is_valid_rti = false;
            tracing::warn!(
                "RTI Access Violation: Cannot read PSR from stack address 0x{:04X}",
                psr_addr.get()
            );
            return;
        }

        // If all checks pass, mark RTI as valid for subsequent phases.
        self.is_valid_rti = true;
        // We don't set MAR here; fetch_operands will handle stack reads.
    }

    fn fetch_operands(&mut self, machine_state: &mut Emulator) -> bool {
        if !self.is_valid_rti {
            return false; // Preconditions failed, skip fetching
        }

        // R6 = SSP
        let ssp = machine_state.r[6].get();

        // LDI-like multi-step fetch:
        // 1. Fetch PC from Mem[SSP]
        // 2. Fetch PSR from Mem[SSP+1]

        // Check if MAR is already set (indicates we might be in the second step)
        let current_mar = machine_state.mar.get();
        let pc_addr = ssp;
        let psr_addr = ssp.wrapping_add(1);

        if current_mar == pc_addr {
            // This means the PC value is now in MDR from the previous cycle's read.
            self.popped_pc = machine_state.mdr; // Store the popped PC

            // Now, set MAR to read the PSR.
            machine_state.mar.set(psr_addr);
            false // We've initiated the second read, no more fetch phases needed.
        } else {
            // --- First Fetch Step ---
            // Set MAR to fetch the PC value from the stack.
            machine_state.mar.set(pc_addr);
            true // Indicate that a second fetch step (triggered implicitly) is needed.
        }
    }

    fn execute_operation(&mut self, machine_state: &mut Emulator) {
        if !self.is_valid_rti {
            return; // Skip execution if preconditions failed or invalid state occurred
        }

        // By this point:
        // - self.popped_pc should hold the PC value (fetched in the first memory cycle).
        // - machine_state.mdr should hold the PSR value (fetched in the second memory cycle).
        self.popped_psr = machine_state.mdr;

        // Update the Supervisor Stack Pointer (R6 = SSP + 2)
        let ssp_val = machine_state.r[6].get();
        machine_state.r[6].set(ssp_val.wrapping_add(2));

        // Restore state from popped PSR
        let new_priv_level = if self.popped_psr.index(15).get() == 1 {
            PrivilegeLevel::User
        } else {
            PrivilegeLevel::Supervisor
        };

        // Check privilege level transition
        let old_priv_level = &machine_state.priv_level();
        if matches!(old_priv_level, PrivilegeLevel::Supervisor)
            && matches!(new_priv_level, PrivilegeLevel::User)
        {
            // --- Switching from Supervisor to User ---
            // Save current R6 (SSP) into saved_ssp
            machine_state.saved_ssp = machine_state.r[6];
            // Restore R6 (USP) from saved_usp
            machine_state.r[6] = machine_state.saved_usp;
        }
        // Note: Switching from User to Supervisor happens during exception/interrupt entry, not RTI.

        // Update PSR after potential stack switch
        machine_state.memory[PSR_ADDR].set(self.popped_psr.get());

        // Restore PC last
        machine_state.pc = self.popped_pc;
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
