use crate::emulator::micro_op::{CycleState, MicroOp, MicroOpGenerator};
use crate::emulator::{
    BitAddressable, EmulatorCell, PrivilegeLevel,
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
                },
                    "
                    if PSR[15] == 1
                        Saved_USP <- R6
                        R6 <- Saved_SSP"
                        .to_owned(),
                ),
                micro_op!(MSG format!("TRAP x{:02X} - switching to supervisor mode", self.trap_vector.get())),
                MicroOp::new_custom(|emu| {
                    emu.set_priv_level(PrivilegeLevel::Supervisor);
                    Ok(())
                }, "PSR[15] <- 0".to_owned()),
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
