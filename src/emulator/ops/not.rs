use crate::emulator::{BitAddressable, Emulator};

use super::Op;
#[derive(Debug)]
pub struct NotOp;

impl Op for NotOp {
    fn prepare_memory_access(&self, _machine_state: &mut Emulator) {
        // NOT doesn't need extra memory access preparation
        tracing::trace!("NOT: No memory access preparation needed");
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("NOT_execute",
            ir = ?machine_state.ir.get()
        );
        let _enter = span.enter();

        // LAYOUT: 1001 | DR | SR | 111111
        let ir = machine_state.ir;
        let dr_index = ir.range(11..9).get() as usize;
        let sr_index = ir.range(8..6).get() as usize;

        // Perform bitwise NOT operation
        let sr_value = machine_state.r[sr_index].get();
        let result = !sr_value;
        tracing::trace!(
            source_register = sr_index,
            source_value = format!("0x{:04X}", sr_value),
            result = format!("0x{:04X}", result),
            "Performing bitwise NOT operation"
        );
        machine_state.r[dr_index].set(result);

        // Update condition codes
        machine_state.update_flags(dr_index);
        tracing::trace!(
            n = machine_state.n.get(),
            z = machine_state.z.get(),
            p = machine_state.p.get(),
            "Updated condition flags after NOT"
        );
    }
}
