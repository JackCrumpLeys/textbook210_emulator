use crate::emulator::micro_op::{CycleState, MicroOp, MicroOpGenerator};
use crate::emulator::{
    area_from_address, BitAddressable, Emulator, EmulatorCell, Exception, PrivilegeLevel, PSR_ADDR,
};
use crate::micro_op;
use std::collections::HashMap;

use super::Op;

#[derive(Debug, Clone)]
/// TRAP works like a special kind of jump instruction.
/// 1. Pushes the current PC (return address) onto the system stack
/// 2. Pushes PSR (processor status register) onto the system stack
/// 3. Switches the CPU to Supervisor mode.
/// 4. Switches the Stack Pointer (R6) from User SP (USP) to Supervisor SP (SSP).
/// 5. Reads the starting address of the trap handler routine from the Trap Vector Table (Memory[0x0000 + ZEXT(trapvect8)]).
/// 6. Jumps to that handler routine address.
pub struct TrapOp {
    pub trap_vector: EmulatorCell, // The 8-bit vector number from the instruction
    pub vector_table_entry_addr: EmulatorCell, // Address in TVT (0x00XX) where handler addr is stored
    pub target_handler_addr: EmulatorCell, // Actual address of the handler routine (read from TVT)
    pub is_valid_read_vector: bool,        // Can we read the entry from the TVT?
    pub is_valid_jump_target: bool,        // Can we jump to the handler address?
}

impl MicroOpGenerator for TrapOp {
    fn generate_plan(&self) -> HashMap<CycleState, Vec<MicroOp>> {
        let mut plan = HashMap::new();

        // Evaluate Address phase - calculate trap vector table address
        plan.insert(
            CycleState::EvaluateAddress,
            vec![micro_op!(Temp <- C(self.trap_vector.get()))],
        );

        // Fetch Operands phase - read handler address from trap vector table
        plan.insert(CycleState::FetchOperands, vec![micro_op!(MAR <- Temp)]);
        // Memory read happens implicitly: MDR <- MEM[MAR] (gets handler address)

        // Execute phase - save state and jump to handler
        plan.insert(
            CycleState::Execute,
            vec![
                micro_op!(Temp <- PSR),
                micro_op!(MSG format!("TRAP x{:02X} - saving user stack pointer", self.trap_vector.get())),
                MicroOp::new_custom(|emu| {
                    if emu.priv_level() == PrivilegeLevel::User {
                        emu.saved_usp = emu.r[6];
                        emu.r[6] = emu.saved_ssp;
                    }
                    Ok(())
                }),
                micro_op!(MSG format!("TRAP x{:02X} - switching to supervisor mode", self.trap_vector.get())),
                MicroOp::new_custom(|emu| {
                    emu.set_priv_level(PrivilegeLevel::Supervisor);
                    Ok(())
                }),
                micro_op!(ALU_OUT <- R(6) + IMM(-1)), // Decrement stack pointer (R6--)
                micro_op!(R(6) <- AluOut),

                micro_op!(MAR <- R(6)),
                micro_op!(MDR <- Temp),
                micro_op!(SET_FLAG(WriteMemory)), // Push PSR onto stack

                // push happens implicitly
                micro_op!(-> Execute),
                micro_op!(ALU_OUT <- R(6) + IMM(-1)), // Decrement stack pointer again
                micro_op!(R(6) <- AluOut),
                micro_op!(MAR <- R(6)),
                micro_op!(MDR <- PC),
                micro_op!(SET_FLAG(WriteMemory)), // Push PC onto stack

                micro_op!(-> Execute), // execute write
                micro_op!(MAR <- C(self.trap_vector.get())),

                micro_op!(-> Execute), // execute read
                micro_op!(PC <- MDR), // Jump to handler (TEMP contains handler address from fetch)
                ],
        );

        plan
    }
}

impl Op for TrapOp {
    fn decode(ir: EmulatorCell) -> Self {
        // LAYOUT: 1111 | 0000 | trapvect8
        let trap_vector = ir.range(7..0); // ZEXT occurs implicitly via range + EmulatorCell

        Self {
            trap_vector,
            vector_table_entry_addr: EmulatorCell::new(0),
            target_handler_addr: EmulatorCell::new(0),
            is_valid_read_vector: false,
            is_valid_jump_target: false,
        }
    }

    fn evaluate_address(&mut self, machine_state: &mut Emulator) {
        // Calculate the address in the Trap Vector Table.
        // Address is 0x0000 + ZEXT(trapvect8).
        let vector_addr_val = self.trap_vector.get(); // Range already zero-extends.
        self.vector_table_entry_addr.set(vector_addr_val);

        // Check if the vector area is within TVT bounds.
        if self.vector_table_entry_addr.get() <= 0x00FF {
            self.is_valid_read_vector = true;
        } else {
            // This indicates a potential issue with the TRAP vector itself or memory setup.
            machine_state.exception = Some(Exception::new_access_control_violation()); // Or a specific Trap exception
            tracing::error!(
                "TRAP Warning: Cannot read Trap Vector Table entry at 0x{:04X}",
                self.vector_table_entry_addr.get()
            );
            debug_assert!(false, "Invalid TRAP vector address");
            self.is_valid_read_vector = false;
        }
    }

    fn fetch_operands(&mut self, machine_state: &mut Emulator) -> bool {
        // If the TVT address is valid, set MAR to read the handler address.
        if self.is_valid_read_vector {
            machine_state.mar = self.vector_table_entry_addr;
        }
        // The actual memory read (MDR <- Mem[MAR]) happens after this phase.
        false // No second fetch phase needed for TRAP itself.
    }

    fn execute_operation(&mut self, machine_state: &mut Emulator) {
        // This phase executes after the memory read triggered by fetch_operands.
        // MDR now holds the target handler address read from the TVT entry.
        if !self.is_valid_read_vector {
            return; // Skip if reading the TVT entry failed.
        }

        // Store the handler address read from MDR.
        self.target_handler_addr = machine_state.mdr;

        // Check if the target handler address is valid to jump to (readable).
        // Again, check as if we are supervisor, since that's the target state.
        let target_area = area_from_address(&self.target_handler_addr);
        if target_area.can_read(&PrivilegeLevel::Supervisor) {
            self.is_valid_jump_target = true;
            let temp = machine_state.memory[PSR_ADDR].get();

            // --- Perform Mode and Stack Switch ---
            // Only switch if not already in Supervisor mode (though TRAP usually comes from User)
            if matches!(machine_state.priv_level(), PrivilegeLevel::User) {
                // Save current R6 (USP) into saved_usp
                machine_state.saved_usp = machine_state.r[6];
                // Load R6 with the Supervisor Stack Pointer (SSP) from saved_ssp
                // Assumes saved_ssp was initialized correctly elsewhere (e.g., OS boot)
                machine_state.r[6] = machine_state.saved_ssp;
            }

            // Set privilege level to Supervisor *after* potential stack swap
            machine_state.set_priv_level(PrivilegeLevel::Supervisor);

            // --- Push PSR onto the stack ---
            // This device stores the current PSR value
            let psr_value = temp;
            // Push PSR onto stack
            let sp_val = machine_state.r[6].get().wrapping_sub(1);
            machine_state.r[6].set(sp_val);
            machine_state.mar.set(sp_val);
            machine_state.mdr.set(psr_value);
            machine_state.write_bit = true;

            // SAFETY: this can only fail if we do not have write access to memory, but we are supervisor
            machine_state.step_write_memory().unwrap(); // pretty botch but this instruction dosent need super detailed cycles

            // --- Push PC onto the stack ---
            // Push PC onto the stack
            let sp_val = machine_state.r[6].get().wrapping_sub(1);
            machine_state.r[6].set(sp_val);
            machine_state.mar.set(sp_val);
            machine_state.mdr = machine_state.pc;
            // Memory write happens in system between cycles

            // --- Update PC ---
            // Set PC to the handler routine address.
            machine_state.pc.set(self.target_handler_addr.get());
        } else {
            // Privilege/Access Violation: The address *read from* the TVT points somewhere invalid.
            machine_state.exception = Some(Exception::new_access_control_violation()); // Or a specific Trap exception
            self.is_valid_jump_target = false;
            tracing::warn!(
                "TRAP Privilege Violation: Handler address 0x{:04X} (from TVT[0x{:04X}]) is not readable/executable.",
                self.target_handler_addr.get(), self.vector_table_entry_addr.get()
            );
        }
    }

    fn store_result(&mut self, machine_state: &mut Emulator) {
        machine_state.write_bit = true; // so we write our pc
    }
}
use std::fmt;

impl fmt::Display for TrapOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let vector_val = self.trap_vector.get(); // Get the 8-bit vector value

        // Check for common trap aliases
        match vector_val {
            0x20 => write!(f, "GETC"),
            0x21 => write!(f, "OUT"),
            0x22 => write!(f, "PUTS"),
            0x23 => write!(f, "IN"),
            0x24 => write!(f, "PUTSP"),
            0x25 => write!(f, "HALT"),
            _ => write!(f, "TRAP x{vector_val:02X}"), // Fallback for unknown vectors
        }?;

        // Add execution state information if available
        if self.is_valid_read_vector {
            if self.target_handler_addr.get() != 0 {
                write!(f, " [handler at x{:04X}]", self.target_handler_addr.get())?;
            } else if self.vector_table_entry_addr.get() != 0 {
                write!(
                    f,
                    " [reading from TVT x{:04X}]",
                    self.vector_table_entry_addr.get()
                )?;
            }
        }

        Ok(())
    }
}
