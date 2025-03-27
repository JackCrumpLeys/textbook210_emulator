use crate::emulator::{BitAddressable, Emulator};

use super::Op;
#[derive(Debug)]
pub struct JsrOp;

impl Op for JsrOp {
    fn prepare_memory_access(&self, _machine_state: &mut Emulator) {
        // JSR doesn't need extra memory access preparation
        tracing::trace!("JSR: No memory access preparation needed");
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("JSR_execute",
            ir = ?machine_state.ir.get(),
            pc = machine_state.pc.get()
        );
        let _enter = span.enter();

        // LAYOUT: 0100 | ToggleBit | rest of bits (PCoffset11 or BaseR)
        let ir = machine_state.ir;
        let curr_pc = machine_state.pc.get();

        // Save return address in R7
        tracing::trace!(
            return_address = format!("0x{:X}", curr_pc),
            "Saving return address in R7"
        );
        machine_state.r[0x7].set(curr_pc);

        // Check if JSR or JSRR
        if ir.index(11).get() == 0x1 {
            // JSR: Use PC-relative addressing
            // Extract and sign-extend PCoffset11
            let pc_offset = ir.range(10..0).sext(10).get();
            tracing::trace!(
                mode = "JSR",
                offset = format!("0x{:X}", pc_offset),
                "PC-relative subroutine jump"
            );

            // PC has already been incremented in fetch, so add the offset directly
            let new_pc = curr_pc.wrapping_add(pc_offset);
            tracing::trace!(
                old_pc = format!("0x{:X}", curr_pc),
                new_pc = format!("0x{:X}", new_pc),
                "Jumping to subroutine"
            );
            machine_state.pc.set(new_pc);
        } else {
            // JSRR: Get address from base register
            let base_r_index = ir.range(8..6).get() as usize;
            tracing::trace!(
                mode = "JSRR",
                base_register = format!("0x{:X}", base_r_index),
                "Register-based subroutine jump"
            );

            let new_pc = machine_state.r[base_r_index].get();
            tracing::trace!(
                old_pc = format!("0x{:X}", curr_pc),
                new_pc = format!("0x{:X}", new_pc),
                from_register = format!("0x{:X}", base_r_index),
                "Jumping to subroutine at address in register"
            );
            machine_state.pc.set(new_pc);
        }
    }
}
