use crate::emulator::micro_op::{CycleState, MicroOp, MicroOpGenerator};
use crate::emulator::{area_from_address, BitAddressable, Emulator, EmulatorCell, Exception};
use crate::micro_op;
use std::collections::HashMap;

use super::Op;

#[derive(Debug, Clone)]
/// load from given register and offset Mem[Base_r + offset6]
pub struct LdrOp {
    pub dr: EmulatorCell,                // Destination Register index
    pub base_r: EmulatorCell,            // Base Register index
    pub offset6: EmulatorCell,           // offset6 (sign-extended)
    pub effective_address: EmulatorCell, // Calculated address
    pub is_valid_load: bool,             // Flag if the address is valid to read from
}

impl MicroOpGenerator for LdrOp {
    fn generate_plan(&self) -> HashMap<CycleState, Vec<MicroOp>> {
        let mut plan = HashMap::new();

        // Evaluate Address phase - calculate effective address from base + offset
        plan.insert(
            CycleState::EvaluateAddress,
            vec![micro_op!(ALU_OUT <- R(self.base_r.get()) + IMM(self.offset6.get() as i16))],
        );

        // Fetch Operands phase - set MAR for memory read
        plan.insert(CycleState::FetchOperands, vec![micro_op!(MAR <- AluOut)]);
        // Memory read MDR <- MEM[MAR] happens implicitly between phases

        // Store Result phase - move loaded value to destination register
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

impl Op for LdrOp {
    fn decode(ir: EmulatorCell) -> Self {
        // LAYOUT: 0110 | DR | BaseR | offset6
        let dr = ir.range(11..9);
        let base_r = ir.range(8..6);
        // Extract and sign-extend offset6
        let offset6 = ir.range(5..0).sext(5);

        Self {
            dr,
            base_r,
            offset6,
            effective_address: EmulatorCell::new(0),
            is_valid_load: false,
        }
    }

    fn evaluate_address(&mut self, machine_state: &mut Emulator) {
        // Calculate effective address: BaseR + SEXT(offset6)
        let base_r_value = machine_state.r[self.base_r.get() as usize];
        let effective_addr_val = base_r_value.get().wrapping_add(self.offset6.get());
        self.effective_address.set(effective_addr_val);

        // Check memory read permissions
        let target_area = area_from_address(&self.effective_address);
        if target_area.can_read(&machine_state.priv_level()) {
            self.is_valid_load = true;
        } else {
            // Privilege violation: Cannot read from this memory location
            machine_state.exception = Some(Exception::new_access_control_violation());
            self.is_valid_load = false;
            tracing::warn!(
                address = format!("0x{:04X}", self.effective_address.get()),
                "LDR Privilege Violation: Cannot read from address"
            );
        }
    }

    fn fetch_operands(&mut self, machine_state: &mut Emulator) -> bool {
        // If the address is valid, set MAR for the implicit memory read.
        if self.is_valid_load {
            machine_state.mar = self.effective_address;
        }
        false
    }

    fn store_result(&mut self, machine_state: &mut Emulator) {
        // Perform the register write if the load address was valid.
        if self.is_valid_load {
            // MDR contains the value implicitly read from memory after fetch_operands.
            let value_loaded = machine_state.mdr;
            let dr_index = self.dr.get() as usize;

            // Write the loaded value into the destination register.
            machine_state.r[dr_index] = value_loaded;

            // Update condition codes based on the value written.
            machine_state.update_flags(dr_index);
        }
        // If !is_valid_load, an exception was set, and the store is skipped.
    }
}
use std::fmt;

impl fmt::Display for LdrOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dr_index = self.dr.get();
        let base_r_index = self.base_r.get();
        // offset6 is already sign-extended from decode
        let offset_val_signed = self.offset6.get() as i16;
        let offset_val_raw = self.offset6.get() & 0x3F; // Get raw 6 bits

        write!(
            f,
            "LDR R{dr_index}, R{base_r_index}, #{offset_val_signed} (x{offset_val_raw:02X})"
        )?;

        if self.is_valid_load {
            write!(f, " [loading")?;
            if self.effective_address.get() != 0 {
                write!(f, " from x{:04X}", self.effective_address.get())?;
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}
