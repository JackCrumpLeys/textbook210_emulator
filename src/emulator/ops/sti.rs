use crate::emulator::{BitAddressable, Emulator};

use super::Op;
#[derive(Debug)]
pub struct StiOp;

impl Op for StiOp {
    fn prepare_memory_access(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("STI_prepare_memory",
            ir = ?machine_state.ir.get(),
            pc = machine_state.pc.get()
        );
        let _enter = span.enter();

        // LAYOUT: 1011 | SR | PCoffset9
        let ir = machine_state.ir;

        // Calculate address of pointer
        let pc_offset = ir.range(8..0).sext(8).get();
        let curr_pc = machine_state.pc.get();
        let pointer_address = curr_pc.wrapping_add(pc_offset);

        tracing::trace!(
            pc = format!("0x{:04X}", curr_pc),
            offset = format!("0x{:04X}", pc_offset),
            pointer_address = format!("0x{:04X}", pointer_address),
            "Calculating pointer address for indirect store"
        );

        // Set MAR to the pointer address
        machine_state.mar.set(pointer_address);
        tracing::trace!(
            mar = format!("0x{:04X}", pointer_address),
            "Setting MAR to pointer address"
        );

        // The memory access system will load MDR with the contents at MAR
        // after this function completes
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("STI_execute",
            ir = ?machine_state.ir.get(),
            mar = machine_state.mar.get(),
            mdr = machine_state.mdr.get()
        );
        let _enter = span.enter();

        // At this point, MDR contains the value from memory at the pointer address
        let ir = machine_state.ir;
        let sr_index = ir.range(11..9).get() as usize;
        let pointer_address = machine_state.mar.get();
        let indirect_address = machine_state.mdr.get();

        tracing::trace!(
            pointer_address = format!("0x{:04X}", pointer_address),
            indirect_address = format!("0x{:04X}", indirect_address),
            "Pointer value loaded from memory"
        );

        // Get the indirect address from MDR and set the MAR to it
        machine_state.mar.set(indirect_address);
        tracing::trace!(
            mar = format!("0x{:04X}", indirect_address),
            "Setting MAR to indirect address for store"
        );

        // Set MDR to the value we want to store
        let sr_value = machine_state.r[sr_index].get();
        machine_state.mdr.set(sr_value);
        tracing::trace!(
            source_register = sr_index,
            value = format!("0x{:04X}", sr_value),
            "Setting MDR to value from register"
        );

        // Store value from MDR to memory at the indirect address in MAR
        let address = machine_state.mar.get() as usize;
        tracing::trace!(
            address = format!("0x{:04X}", address),
            value = format!("0x{:04X}", sr_value),
            "Storing value in memory at indirect address"
        );
        machine_state.memory[address].set(machine_state.mdr.get());
    }
}
