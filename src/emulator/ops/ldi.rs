use crate::emulator::micro_op::{CycleState, MicroOp, MicroOpGenerator};
use crate::emulator::{area_from_address, BitAddressable, Emulator, EmulatorCell, Exception};
use crate::micro_op;
use std::collections::HashMap;

use super::Op;

#[derive(Debug, Clone)]
/// Load indirectly from an offset so we load Mem[Mem[PC + PCoffset9]]
pub struct LdiOp {
    pub dr: EmulatorCell,               // Destination Register index
    pub pc_offset: EmulatorCell,        // PCoffset9 (sign-extended)
    pub pointer_address: EmulatorCell,  // Address containing the final address
    pub indirect_address: EmulatorCell, // The final address loaded from pointer_address
    pub is_valid_load_step1: bool,      // Flag if pointer_address is valid to read from
    pub is_valid_load_step2: bool,      // Flag if indirect_address is valid to read from
}

impl MicroOpGenerator for LdiOp {
    fn generate_plan(&self) -> HashMap<CycleState, Vec<MicroOp>> {
        let mut plan = HashMap::new();

        // Evaluate Address phase - calculate pointer address
        plan.insert(
            CycleState::EvaluateAddress,
            vec![micro_op!(ALU_OUT <- PC + PCOFFSET(self.pc_offset.get() as i16))],
        );

        // Fetch Operands phase - first memory read for pointer
        plan.insert(
            CycleState::FetchOperands,
            vec![
                micro_op!(MAR <- AluOut),
                // First memory read happens implicitly: MDR <- MEM[MAR]
                micro_op!(-> Execute),
                micro_op!(MAR <- MDR),
            ],
        );
        // Then second fetch happens with MDR as new address

        // Store Result phase - move final loaded value to destination register
        plan.insert(
            CycleState::StoreResult,
            vec![
                micro_op!(R(self.dr.get()) <- MDR),
                micro_op!(SET_CC(self.dr.get())),
            ],
        );

        plan
    }
}

impl Op for LdiOp {
    fn decode(ir: EmulatorCell) -> Self {
        // LAYOUT: 1010 | DR | PCoffset9
        let dr = ir.range(11..9);
        // Extract and sign-extend PCoffset9
        let pc_offset = ir.range(8..0).sext(8);

        Self {
            dr,
            pc_offset,
            pointer_address: EmulatorCell::new(0),
            indirect_address: EmulatorCell::new(0),
            is_valid_load_step1: false,
            is_valid_load_step2: false,
        }
    }
}

use std::fmt;

impl fmt::Display for LdiOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format offset as signed decimal
        let offset_val = self.pc_offset.get() as i16; // Cast to signed for proper display

        write!(
            f,
            "LDI R{}, #{} (x{:03X})",
            self.dr.get(),
            offset_val,
            self.pc_offset.get() & 0x1FF // Mask to 9 bits for hex
        )?;

        if self.is_valid_load_step1 && self.is_valid_load_step2 {
            write!(
                f,
                " [taking mem[mem[{:04X}]] = mem[{:04X}]]",
                self.pointer_address.get(),
                self.indirect_address.get()
            )?;
        }
        Ok(())
    }
}
