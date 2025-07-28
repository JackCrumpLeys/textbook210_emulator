use crate::emulator::micro_op::{CycleState, MicroOp, MicroOpGenerator};
use crate::emulator::{BitAddressable, EmulatorCell};
use crate::micro_op;
use std::collections::HashMap;

use super::Op;

#[derive(Debug, Clone)]
/// Store some register value at adress of some other register value
pub struct StrOp {
    pub sr: EmulatorCell,                // Source Register index
    pub base_r: EmulatorCell,            // Base Register index
    pub offset6: EmulatorCell,           // offset6 (sign-extended)
    pub effective_address: EmulatorCell, // Calculated address
    pub value_to_store: EmulatorCell,    // Value from SR (fetched in fetch_operands)
    pub is_valid_store: bool,            // Flag if the address is valid to write to
}

impl MicroOpGenerator for StrOp {
    fn generate_plan(&self) -> HashMap<CycleState, Vec<MicroOp>> {
        let mut plan = HashMap::new();

        // Evaluate Address phase - calculate effective address from base + offset
        plan.insert(
            CycleState::EvaluateAddress,
            vec![micro_op!(ALU_OUT <- R(self.base_r.get()) + IMM(self.offset6.get() as i16))],
        );

        // Store Result phase - trigger memory write
        plan.insert(
            CycleState::StoreResult,
            vec![
                micro_op!(MAR <- AluOut),
                micro_op!(MDR <- R(self.sr.get())),
                micro_op!(SET_FLAG(WriteMemory)),
            ],
        );

        plan
    }
}

impl Op for StrOp {
    fn decode(ir: EmulatorCell) -> Self {
        // LAYOUT: 0111 | SR | BaseR | offset6
        let sr = ir.range(11..9);
        let base_r = ir.range(8..6);
        // Extract and sign-extend offset6
        let offset6 = ir.range(5..0).sext(5);

        Self {
            sr,
            base_r,
            offset6,
            effective_address: EmulatorCell::new(0),
            value_to_store: EmulatorCell::new(0),
            is_valid_store: false,
        }
    }
}
use std::fmt;

impl fmt::Display for StrOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Get the raw 6-bit offset value
        let offset_val_6bit = self.offset6.get() & 0x3F;
        // Calculate the sign-extended value (as i16 for display)
        let offset_val_sext = if (offset_val_6bit >> 5) & 1 == 1 {
            // Negative number, extend with 1s
            (offset_val_6bit as i16) | !0x3F // or (offset_val_6bit as i16) - 64
        } else {
            offset_val_6bit as i16
        };

        write!(
            f,
            "STR R{}, R{}, #{} (x{:02X})",
            self.sr.get(),
            self.base_r.get(),
            offset_val_sext, // Display sign-extended decimal
            offset_val_6bit  // Display raw 6-bit hex
        )?;

        if self.is_valid_store {
            write!(f, " [Storing")?;
            if self.value_to_store.get() != 0 {
                write!(f, " the value x{:04X}", self.effective_address.get())?;
            }
            if self.effective_address.get() != 0 {
                write!(f, " into x{:04X}", self.effective_address.get())?;
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}
