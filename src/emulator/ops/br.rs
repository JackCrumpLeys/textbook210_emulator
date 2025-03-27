use crate::emulator::{BitAddressable, Emulator};

use super::Op;
#[derive(Debug)]
pub struct BrOp;

impl Op for BrOp {
    fn prepare_memory_access(&self, _machine_state: &mut Emulator) {
        // Branch doesn't need extra memory access preparation
        tracing::trace!("BR: No memory access preparation needed");
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("BR_execute",
            ir = ?machine_state.ir.get(),
            pc = machine_state.pc.get()
        );
        let _enter = span.enter();

        // LAYOUT: 0000 | N | Z | P | PCoffset9
        let ir = machine_state.ir;

        // Extract NZP bits and PCoffset9
        let n_bit = ir.index(11).get();
        let z_bit = ir.index(10).get();
        let p_bit = ir.index(9).get();
        tracing::trace!(
            n = format!("0x{:X}", n_bit),
            z = format!("0x{:X}", z_bit),
            p = format!("0x{:X}", p_bit),
            "Branch condition codes"
        );

        // Extract and sign-extend PCoffset9
        let pc_offset = ir.range(8..0).sext(8).get();
        tracing::trace!(
            offset = format!("0x{:X}", pc_offset),
            "PC offset for branch"
        );

        // Check if condition codes match current state
        let n_match = n_bit == 0x1 && machine_state.n.get() == 0x1;
        let z_match = z_bit == 0x1 && machine_state.z.get() == 0x1;
        let p_match = p_bit == 0x1 && machine_state.p.get() == 0x1;

        tracing::trace!(
            current_n = format!("0x{:X}", machine_state.n.get()),
            current_z = format!("0x{:X}", machine_state.z.get()),
            current_p = format!("0x{:X}", machine_state.p.get()),
            "Current machine condition flags"
        );

        tracing::trace!(
            n_match = n_match,
            z_match = z_match,
            p_match = p_match,
            "Condition code matching results"
        );

        // If any condition matches, branch to the target address
        if n_match || z_match || p_match {
            // PC has already been incremented in fetch, so we add the offset directly
            let old_pc = machine_state.pc.get();
            let new_pc = old_pc.wrapping_add(pc_offset);
            tracing::trace!(
                old_pc = format!("0x{:X}", old_pc),
                new_pc = format!("0x{:X}", new_pc),
                "Taking branch"
            );
            machine_state.pc.set(new_pc);
        } else {
            tracing::trace!("No condition match, not taking branch");
        }
        // If no condition matches, execution continues normally
    }
}
