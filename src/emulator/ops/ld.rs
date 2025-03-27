use crate::emulator::{BitAddressable, Emulator};

use super::Op;
#[derive(Debug)]
pub struct LdOp;

impl Op for LdOp {
    fn prepare_memory_access(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("LD_prepare_memory",
            ir = ?machine_state.ir.get(),
            pc = machine_state.pc.get()
        );
        let _enter = span.enter();

        // LAYOUT: 0010 | DR | PCoffset9
        let ir = machine_state.ir;

        // Calculate effective address
        let pc_offset = ir.range(8..0).sext(8).get();
        let curr_pc = machine_state.pc.get();
        let effective_address = curr_pc.wrapping_add(pc_offset);

        tracing::trace!(
            pc = format!("0x{:X}", curr_pc),
            offset = format!("0x{:X}", pc_offset),
            effective_address = format!("0x{:X}", effective_address),
            "Calculating effective address for load"
        );

        // Set MAR to the effective address
        machine_state.mar.set(effective_address);
        tracing::trace!(
            mar = format!("0x{:X}", effective_address),
            "Setting MAR for memory access"
        );
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("LD_execute",
            ir = ?machine_state.ir.get(),
            mar = machine_state.mar.get(),
            mdr = machine_state.mdr.get()
        );
        let _enter = span.enter();

        // LAYOUT: 0010 | DR | PCoffset9
        let ir = machine_state.ir;
        let dr_index = ir.range(11..9).get() as usize;
        let effective_address = machine_state.mar.get();

        // MDR was loaded during memory access phase
        let value = machine_state.mdr.get();
        tracing::trace!(
            address = format!("0x{:X}", effective_address),
            value = format!("0x{:X}", value),
            dest_register = format!("0x{:X}", dr_index),
            "Loading value from memory into register"
        );
        machine_state.r[dr_index].set(value);

        // Update condition codes
        machine_state.update_flags(dr_index);
        tracing::trace!(
            n = format!("0x{:X}", machine_state.n.get()),
            z = format!("0x{:X}", machine_state.z.get()),
            p = format!("0x{:X}", machine_state.p.get()),
            "Updated condition flags after load"
        );
    }
}
