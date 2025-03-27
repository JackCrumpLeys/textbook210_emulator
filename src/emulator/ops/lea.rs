use crate::emulator::{BitAddressable, Emulator};

use super::Op;
#[derive(Debug)]
pub struct LeaOp;

impl Op for LeaOp {
    fn prepare_memory_access(&self, _machine_state: &mut Emulator) {
        // LEA doesn't need extra memory access preparation
        tracing::trace!("LEA: No memory access preparation needed");
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("LEA_execute",
            ir = ?machine_state.ir.get(),
            pc = machine_state.pc.get()
        );
        let _enter = span.enter();

        // LAYOUT: 1110 | DR | PCoffset9
        let ir = machine_state.ir;
        let dr_index = ir.range(11..9).get() as usize;

        // Calculate effective address (PC + PCoffset9)
        let pc_offset = ir.range(8..0).sext(8).get();
        let curr_pc = machine_state.pc.get();
        let effective_address = curr_pc.wrapping_add(pc_offset);

        tracing::trace!(
            pc = format!("0x{:04X}", curr_pc),
            offset = format!("0x{:04X}", pc_offset),
            effective_address = format!("0x{:04X}", effective_address),
            "Calculating effective address for load effective address"
        );

        // Load effective address into DR
        tracing::trace!(
            address = format!("0x{:04X}", effective_address),
            dest_register = dr_index,
            "Loading effective address into register"
        );
        machine_state.r[dr_index].set(effective_address);

        // Update condition codes
        machine_state.update_flags(dr_index);
        tracing::trace!(
            n = machine_state.n.get(),
            z = machine_state.z.get(),
            p = machine_state.p.get(),
            "Updated condition flags after load effective address"
        );
    }
}
