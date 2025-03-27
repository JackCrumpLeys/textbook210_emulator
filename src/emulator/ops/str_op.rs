use crate::emulator::{BitAddressable, Emulator};

use super::Op;
#[derive(Debug)]
pub struct StrOp;

impl Op for StrOp {
    fn prepare_memory_access(&self, _machine_state: &mut Emulator) {
        // STR doesn't need to prepare memory access
        tracing::trace!("STR: Memory access preparation handled in execute phase");
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("STR_execute",
            ir = ?machine_state.ir.get()
        );
        let _enter = span.enter();

        // LAYOUT: 0111 | SR | BaseR | offset6
        let ir = machine_state.ir;
        let sr_index = ir.range(11..9).get() as usize;
        let base_r_index = ir.range(8..6).get() as usize;

        // Calculate effective address: BaseR + offset6
        let offset = ir.range(5..0).sext(5).get();
        let base_r_value = machine_state.r[base_r_index].get();
        let effective_address = base_r_value.wrapping_add(offset);

        tracing::trace!(
            base_register = base_r_index,
            base_value = format!("0x{:04X}", base_r_value),
            offset = format!("0x{:04X}", offset),
            effective_address = format!("0x{:04X}", effective_address),
            "Calculating effective address for register-relative store"
        );

        // Set MAR to the effective address and MDR to the value to store
        machine_state.mar.set(effective_address);
        let sr_value = machine_state.r[sr_index].get();
        machine_state.mdr.set(sr_value);
        tracing::trace!(
            mar = format!("0x{:04X}", effective_address),
            mdr = format!("0x{:04X}", sr_value),
            source_register = sr_index,
            "Setting memory registers for store operation"
        );

        // Store value from MDR to memory at address in MAR
        let address = machine_state.mar.get() as usize;
        tracing::trace!(
            address = format!("0x{:04X}", address),
            value = format!("0x{:04X}", sr_value),
            "Storing value in memory at register-relative address"
        );
        machine_state.memory[address].set(machine_state.mdr.get());
    }
}
