use crate::emulator::micro_op::{CycleState, MicroOp, MicroOpGenerator};
use crate::emulator::{BitAddressable, EmulatorCell};
use crate::micro_op;
use std::collections::HashMap;

use super::Op;

#[derive(Debug, Clone)]
/// Store some register value at `mem[mem[pc+pc_offset]]`
pub struct StiOp {
    pub sr: EmulatorCell,               // Source Register index
    pub pc_offset: EmulatorCell,        // PCoffset9 (sign-extended)
    pub pointer_address: EmulatorCell,  // Address containing the final address
    pub indirect_address: EmulatorCell, // The final address loaded from pointer_address (set in execute)
    pub value_to_store: EmulatorCell,   // Value from SR to be stored (set in execute)
    pub is_valid_read_step1: bool,      // Flag if pointer_address is valid to read from
    pub is_valid_write_step2: bool, // Flag if indirect_address is valid to write to (set in execute)
}

impl MicroOpGenerator for StiOp {
    fn generate_plan(&self) -> HashMap<CycleState, Vec<MicroOp>> {
        let mut plan = HashMap::new();

        // Evaluate Address phase - calculate pointer address
        plan.insert(
            CycleState::EvaluateAddress,
            vec![micro_op!(ALU_OUT <- PC + PCOFFSET(self.pc_offset.get() as i16))],
        );

        // Fetch Operands phase - read pointer to get final address
        plan.insert(CycleState::FetchOperands, vec![micro_op!(MAR <- AluOut)]);
        // Memory read happens implicitly: MDR <- MEM[MAR] (gets indirect address)

        // Store Result phase - trigger memory write
        plan.insert(
            CycleState::StoreResult,
            vec![
                micro_op!(MAR <- MDR),              // Set MAR to indirect address
                micro_op!(MDR <- R(self.sr.get())), // Load value to store
                micro_op!(SET_FLAG(WriteMemory)),
            ],
        );

        plan
    }
}

impl Op for StiOp {
    fn decode(ir: EmulatorCell) -> Self {
        // LAYOUT: 1011 | SR | PCoffset9

        // Extract and sign-extend PCoffset9
        let pc_offset = ir.range(8..0).sext(8);

        Self {
            sr: ir.range(11..9),
            pc_offset,
            pointer_address: EmulatorCell::new(0),
            indirect_address: EmulatorCell::new(0),
            value_to_store: EmulatorCell::new(0),
            is_valid_read_step1: false,
            is_valid_write_step2: false,
        }
    }
}
use std::fmt;

impl fmt::Display for StiOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display based on the state after decode (sr and pc_offset are known)
        // indirect_address, value_to_store, etc., are determined later
        // and are not part of the basic instruction format.

        let sr_index = self.sr.get();
        // Sign-extend the 9-bit offset for correct decimal representation
        let offset_val = self.pc_offset.sext(8).get() as i16;

        write!(
            f,
            "STI R{}, #{} (x{:03X})",
            sr_index,
            offset_val,                   // Display signed decimal offset
            self.pc_offset.get() & 0x1FF  // Display raw 9-bit hex offset
        )?;

        if self.is_valid_read_step1 && self.is_valid_write_step2 {
            write!(
                f,
                " [storing {:04X} to mem[mem[{:04X}]] = mem[{:04X}]]",
                self.value_to_store.get(),
                self.pointer_address.get(),
                self.indirect_address.get()
            )?;
        }
        Ok(())
    }
}
