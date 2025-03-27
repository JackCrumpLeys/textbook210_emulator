use crate::emulator::{BitAddressable, Emulator};

use super::Op;
#[derive(Debug)]
pub struct JmpOp;

impl Op for JmpOp {
    fn prepare_memory_access(&self, _machine_state: &mut Emulator) {
        // JMP doesn't need extra memory access preparation
        tracing::trace!("JMP: No memory access preparation needed");
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("JMP_execute",
            ir = ?machine_state.ir.get(),
            pc = machine_state.pc.get()
        );
        let _enter = span.enter();

        // LAYOUT: 1100 | 000 | BaseR | 000000
        let ir = machine_state.ir;

        // Extract base register index
        let base_r_index = ir.range(8..6).get() as usize;
        tracing::trace!(
            base_register = format!("0x{:X}", base_r_index),
            "Using base register for jump"
        );

        // Set PC to the value in the base register
        let old_pc = machine_state.pc.get();
        let new_pc = machine_state.r[base_r_index].get();
        tracing::trace!(
            old_pc = format!("0x{:X}", old_pc),
            new_pc = format!("0x{:X}", new_pc),
            from_register = format!("0x{:X}", base_r_index),
            "Jumping to address in register"
        );
        machine_state.pc.set(new_pc);

        // Note: RET is a special case of JMP where BaseR is R7
        if base_r_index == 0x7 {
            tracing::trace!("This is a RET instruction (JMP R7)");
        }
    }
}
