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

    fn evaluate_address(&mut self, machine_state: &mut Emulator) {
        // Phase 1: Calculate and validate the address of the pointer.
        // PC was already incremented during the fetch phase
        let current_pc = machine_state.pc;
        let pointer_addr_val = current_pc.get().wrapping_add(self.pc_offset.get());
        self.pointer_address.set(pointer_addr_val);

        // Check memory read permissions for the pointer address
        let pointer_area = area_from_address(&self.pointer_address);
        if pointer_area.can_read(&machine_state.priv_level()) {
            self.is_valid_load_step1 = true;
        } else {
            // Privilege violation: Cannot read the pointer address
            machine_state.exception = Some(Exception::new_access_control_violation());
            self.is_valid_load_step1 = false;
            tracing::warn!(
                address = format!("0x{:04X}", self.pointer_address.get()),
                "LDI Privilege Violation (Step 1): Cannot read pointer address"
            );
        }
    }

    // fetch_operands is called twice for LDI.
    // 1. First call: Set MAR to pointer_address. Return true.
    // 2. Second call (after first memory read): MDR holds indirect_address. Set MAR to indirect_address. Return false.
    fn fetch_operands(&mut self, machine_state: &mut Emulator) -> bool {
        if self.is_valid_load_step1 {
            // Determine if this is the first or second fetch phase for this LDI
            // We can infer this based on whether indirect_address has been set (which happens in execute_op)

            // If MAR is currently 0 or doesn't match pointer_address, it's likely the first fetch.
            if machine_state.mar.get() != self.pointer_address.get() {
                // --- First Fetch Phase ---
                machine_state.mar = self.pointer_address;
                // Indicate that a second fetch phase (after memory read) is needed.
                true
            } else {
                // --- Second Fetch Phase ---
                // MDR should now hold the indirect address from the first read.
                self.indirect_address = machine_state.mdr;

                // Check permissions for the indirect address before setting MAR again.
                let indirect_area = area_from_address(&self.indirect_address);
                if indirect_area.can_read(&machine_state.priv_level()) {
                    self.is_valid_load_step2 = true;
                    // Set MAR for the final memory read.
                    machine_state.mar = self.indirect_address;
                } else {
                    // Privilege violation: Cannot read from the final indirect address
                    machine_state.exception = Some(Exception::new_access_control_violation());
                    self.is_valid_load_step2 = false;
                    tracing::warn!(
                        address = format!("0x{:04X}", self.indirect_address.get()),
                        "LDI Privilege Violation (Step 2): Cannot read final indirect address"
                    );
                    machine_state.mar = EmulatorCell::new(0); // Clear MAR on error
                }
                // No more fetch phases needed after this.
                false
            }
        } else {
            // First step failed (invalid pointer address), no fetch needed.
            false
        }
    }

    fn store_result(&mut self, machine_state: &mut Emulator) {
        // This phase occurs after the *second* memory read (triggered by the second fetch_operands).
        // Store the final value (now in MDR) into DR.
        if self.is_valid_load_step1 && self.is_valid_load_step2 {
            // MDR contains the final value read from memory
            let final_value = machine_state.mdr;
            let dr_index = self.dr.get() as usize;

            // Write the final value into the destination register
            machine_state.r[dr_index] = final_value;

            // Update condition codes based on the value written to the register
            machine_state.update_flags(dr_index);
        }
        // If either step failed, an exception was set, and the store is skipped.
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
