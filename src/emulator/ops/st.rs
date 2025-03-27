use crate::emulator::{BitAddressable, Emulator};

use super::Op;
#[derive(Debug)]
pub struct StOp;

impl Op for StOp {
    fn prepare_memory_access(&self, _machine_state: &mut Emulator) {
        // ST doesn't need to prepare memory access as we handle it in execute
        tracing::trace!("ST: Memory access preparation handled in execute phase");
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("ST_execute",
            ir = ?machine_state.ir.get(),
            pc = machine_state.pc.get()
        );
        let _enter = span.enter();

        // LAYOUT: 0011 | SR | PCoffset9
        let ir = machine_state.ir;
        let sr_index = ir.range(11..9).get() as usize;

        // Calculate effective address
        let pc_offset = ir.range(8..0).sext(8).get();
        let curr_pc = machine_state.pc.get();
        let effective_address = curr_pc.wrapping_add(pc_offset);

        tracing::trace!(
            pc = format!("0x{:04X}", curr_pc),
            offset = format!("0x{:04X}", pc_offset),
            effective_address = format!("0x{:04X}", effective_address),
            "Calculating effective address for store"
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
            "Storing value in memory"
        );
        machine_state.memory[address].set(machine_state.mdr.get());
    }
}
