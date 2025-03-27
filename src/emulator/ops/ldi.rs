use crate::emulator::{BitAddressable, Emulator};

use super::Op;
#[derive(Debug)]
pub struct LdiOp;

impl Op for LdiOp {
    fn prepare_memory_access(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("LDI_prepare_memory",
            ir = ?machine_state.ir.get(),
            pc = machine_state.pc.get()
        );
        let _enter = span.enter();

        // LAYOUT: 1010 | DR | PCoffset9
        let ir = machine_state.ir;

        // Calculate address of pointer
        let pc_offset = ir.range(8..0).sext(8).get();
        let curr_pc = machine_state.pc.get();
        let pointer_address = curr_pc.wrapping_add(pc_offset);

        tracing::trace!(
            pc = format!("0x{:04X}", curr_pc),
            offset = format!("0x{:04X}", pc_offset),
            pointer_address = format!("0x{:04X}", pointer_address),
            "Calculating pointer address for indirect load"
        );

        // Set MAR to the pointer address
        machine_state.mar.set(pointer_address);
        tracing::trace!(
            mar = format!("0x{:04X}", pointer_address),
            "Setting MAR to pointer address"
        );

        // Note: The actual memory read (MAR -> MDR) will happen after this function completes
        // The memory system will load the pointer value from machine_state.memory[MAR] into MDR
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("LDI_execute",
            ir = ?machine_state.ir.get(),
            mar = machine_state.mar.get(),
            mdr = machine_state.mdr.get()
        );
        let _enter = span.enter();

        // LAYOUT: 1010 | DR | PCoffset9
        let ir = machine_state.ir;
        let dr_index = ir.range(11..9).get() as usize;

        // Get the indirect address from MDR and set MAR to it
        let pointer_value = machine_state.mdr.get();
        machine_state.mar.set(pointer_value);
        tracing::trace!(
            pointer_value = format!("0x{:04X}", pointer_value),
            "Setting MAR to indirect address for final load"
        );

        machine_state
            .mdr
            .set(machine_state.memory[machine_state.mar.get() as usize].get());

        let value = machine_state.mdr.get();
        tracing::trace!(
            indirect_address = format!("0x{:04X}", machine_state.mar.get()),
            value = format!("0x{:04X}", value),
            dest_register = dr_index,
            "Loading value from indirect address into register"
        );
        machine_state.r[dr_index].set(value);

        // Update condition codes
        machine_state.update_flags(dr_index);
        tracing::trace!(
            n = machine_state.n.get(),
            z = machine_state.z.get(),
            p = machine_state.p.get(),
            "Updated condition flags after indirect load"
        );
    }
}
